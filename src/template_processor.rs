use crate::types::*;
use anyhow;
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};
use serde_json;
use std::{fs, path::Path};
use regex;

// Embed the templates directory at compile time
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/templates");

pub fn get_available_templates() -> anyhow::Result<Vec<String>> {
    let mut templates = Vec::new();
    
    for entry in TEMPLATES_DIR.dirs() {
        templates.push(entry.path().file_name().unwrap().to_str().unwrap().to_string());
    }
    
    templates.sort();
    Ok(templates)
}

pub fn load_template_metadata(template_name: &str) -> anyhow::Result<Option<TemplateMetadata>> {
    if let Some(_template_dir) = TEMPLATES_DIR.get_dir(template_name) {
        if let Some(metadata_file) = TEMPLATES_DIR.get_file(&format!("{}/template.toml", template_name)) {
            let content = metadata_file.contents_utf8()
                .ok_or_else(|| anyhow::anyhow!("template.toml is not valid UTF-8"))?;
            let metadata: TemplateMetadata = toml::from_str(content)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    } else {
        anyhow::bail!("Template '{}' not found", template_name);
    }
}

pub fn setup_handlebars() -> Handlebars<'static> {
    let handlebars = Handlebars::new();
    
    // Handlebars 6.x has built-in comparison helpers: eq, ne, gt, gte, lt, lte
    // and logical helpers: and, or, not - no need to register custom ones

    handlebars
}

pub fn process_template_directory(
    template_name: &str,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
) -> anyhow::Result<()> {
    if let Some(template_dir) = TEMPLATES_DIR.get_dir(template_name) {
        process_embedded_template_directory(template_dir, output_dir, handlebars, data, metadata, "")
    } else {
        anyhow::bail!("Template '{}' not found", template_name);
    }
}

fn process_embedded_template_directory(
    template_dir: &include_dir::Dir,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
    relative_path: &str,
) -> anyhow::Result<()> {
    // Create the output directory
    fs::create_dir_all(output_dir)?;
    
    // Process all files in the embedded directory
    for file in template_dir.files() {
        let file_name = file.path().file_name().unwrap().to_str().unwrap();
        let current_relative_path = if relative_path.is_empty() {
            file_name.to_string()
        } else {
            format!("{}/{}", relative_path, file_name)
        };
        
        // Check if this file should be included based on conditions
        if should_skip_file(metadata, &current_relative_path, data) {
            continue;
        }
        
        // Check if this file should be templated
        let file_config = get_file_config(metadata, &current_relative_path);
        let should_template = file_config.map_or(true, |fc| fc.template);
        
        // Template the filename if needed
        let output_filename = if should_template {
            handlebars.render_template(file_name, data)?
        } else {
            file_name.to_string()
        };
        
        let output_path = output_dir.join(&output_filename);
        
        // Get file content
        let content = if let Some(utf8_content) = file.contents_utf8() {
            if should_template {
                let rendered = handlebars.render_template(utf8_content, data)?;
                // Skip empty files (allows Handlebars conditionals to hide entire files)
                if rendered.trim().is_empty() {
                    continue;
                }
                rendered
            } else {
                utf8_content.to_string()
            }
        } else {
            // Binary file - write as-is
            fs::write(&output_path, file.contents())?;
            continue;
        };
        
        // Write the processed content
        fs::write(&output_path, content)?;
        
        // Set executable permissions if needed
        if file_config.map_or(false, |fc| fc.executable) {
            set_executable(&output_path)?;
        }
    }
    
    // Process subdirectories
    for subdir in template_dir.dirs() {
        let subdir_name = subdir.path().file_name().unwrap().to_str().unwrap();
        let current_relative_path = if relative_path.is_empty() {
            subdir_name.to_string()
        } else {
            format!("{}/{}", relative_path, subdir_name)
        };
        
        // Check directory inclusion rules
        if let Some(metadata) = metadata {
            // Check if this directory should be excluded in minimal mode
            if data.get("minimal").and_then(|v| v.as_bool()).unwrap_or(false) {
                if metadata.file_conventions.full_mode_only.contains(&subdir_name.to_string()) {
                    continue;
                }
            }
        }
        
        // Template the directory name if needed
        let output_dirname = handlebars.render_template(subdir_name, data)?;
        let output_subdir = output_dir.join(&output_dirname);
        
        process_embedded_template_directory(subdir, &output_subdir, handlebars, data, metadata, &current_relative_path)?;
    }
    
    Ok(())
}

fn get_file_config<'a>(metadata: Option<&'a TemplateMetadata>, relative_path: &str) -> Option<&'a FileConfig> {
    metadata?.files.iter().find(|f| f.path == relative_path)
}

fn should_skip_file(metadata: Option<&TemplateMetadata>, relative_path: &str, data: &serde_json::Value) -> bool {
    !should_include_file(metadata, relative_path, data)
}

fn should_include_file(metadata: Option<&TemplateMetadata>, relative_path: &str, data: &serde_json::Value) -> bool {
    // First check explicit file configuration
    if let Some(config) = get_file_config(metadata, relative_path) {
        if let Some(condition) = &config.condition {
            return evaluate_condition(condition, data);
        }
    }
    
    // Apply convention-based rules
    if let Some(metadata) = metadata {
        return should_include_by_convention(&metadata.file_conventions, relative_path, data);
    }
    
    true // Include by default if no metadata
}

fn should_include_by_convention(conventions: &FileConventions, relative_path: &str, data: &serde_json::Value) -> bool {
    // Check if file is in always-included directories
    for always_dir in &conventions.always_include {
        if relative_path.starts_with(always_dir) {
            return true;
        }
    }
    
    // Check if file should be excluded in minimal mode
    let is_minimal = data.get("minimal").and_then(|v| v.as_bool()).unwrap_or(false);
    if is_minimal {
        for full_only_dir in &conventions.full_mode_only {
            if relative_path.starts_with(full_only_dir) {
                return false;
            }
        }
    }
    
    true // Include by default
}

// Evaluate condition using Handlebars built-in helpers
fn evaluate_condition(condition: &str, data: &serde_json::Value) -> bool {
    let handlebars = setup_handlebars();
    
    // Convert condition to Handlebars template format
    let handlebars_condition = convert_condition_to_handlebars(condition);
    let template = format!("{{{{#if {}}}}}true{{{{/if}}}}", handlebars_condition);
    
    match handlebars.render_template(&template, data) {
        Ok(result) => result.trim() == "true",
        Err(_) => false, // Default to false if condition evaluation fails
    }
}

fn convert_condition_to_handlebars(condition: &str) -> String {
    // Handle AND conditions first (higher precedence)
    if condition.contains("&&") {
        let parts: Vec<String> = condition.split("&&")
            .map(|part| convert_condition_to_handlebars(part.trim()))
            .collect();
        return format!("(and {})", parts.join(" "));
    }
    
    // Handle OR conditions
    if condition.contains("||") {
        let parts: Vec<String> = condition.split("||")
            .map(|part| convert_condition_to_handlebars(part.trim()))
            .collect();
        return format!("(or {})", parts.join(" "));
    }
    
    // Handle single conditions
    convert_single_condition_to_handlebars(condition)
}

fn convert_single_condition_to_handlebars(condition: &str) -> String {
    // Handle string equality: "integration == 'value'" -> "(eq integration 'value')"
    if let Some(captures) = regex::Regex::new(r"(\w+)\s*==\s*'([^']+)'").unwrap().captures(condition) {
        let var_name = captures.get(1).unwrap().as_str();
        let value = captures.get(2).unwrap().as_str();
        return format!("(eq {} '{}')", var_name, value);
    }
    
    // Handle boolean checks: "minimal == false" -> "(eq minimal false)"
    if let Some(captures) = regex::Regex::new(r"(\w+)\s*==\s*(true|false)").unwrap().captures(condition) {
        let var_name = captures.get(1).unwrap().as_str();
        let bool_value = captures.get(2).unwrap().as_str();
        return format!("(eq {} {})", var_name, bool_value);
    }
    
    // Unknown condition format, return something that evaluates to false
    "false".to_string()
}

fn set_executable(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        fs::set_permissions(path, permissions)?;
    }
    
    #[cfg(not(unix))]
    {
        // On non-Unix systems, executable permissions are typically not needed
        // Files like .bat, .cmd, .exe are executable by extension
    }
    
    Ok(())
}