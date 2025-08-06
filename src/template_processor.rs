use crate::types::*;
use anyhow;
use handlebars::Handlebars;
use regex::{Regex, RegexBuilder};
use serde_json;
use std::{fs, path::Path, process::Command};

// Conditional template loading based on build mode
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};

#[cfg(not(debug_assertions))]
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/templates");

#[cfg(debug_assertions)]
static TEMPLATES_PATH: &str = "src/templates";

pub fn get_available_templates() -> anyhow::Result<Vec<String>> {
    let mut templates = Vec::new();

    #[cfg(not(debug_assertions))]
    {
        // Release mode: use embedded templates
        for entry in TEMPLATES_DIR.dirs() {
            templates.push(
                entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
        }
    }

    #[cfg(debug_assertions)]
    {
        // Debug mode: read from filesystem
        let templates_path = Path::new(TEMPLATES_PATH);
        if templates_path.exists() {
            for entry in fs::read_dir(templates_path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        templates.push(name.to_string());
                    }
                }
            }
        }
    }

    templates.sort();
    Ok(templates)
}

pub fn load_template_metadata(template_name: &str) -> anyhow::Result<Option<TemplateMetadata>> {
    #[cfg(not(debug_assertions))]
    {
        // Release mode: use embedded templates
        if let Some(_template_dir) = TEMPLATES_DIR.get_dir(template_name) {
            if let Some(metadata_file) =
                TEMPLATES_DIR.get_file(&format!("{}/template.toml", template_name))
            {
                let content = metadata_file
                    .contents_utf8()
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

    #[cfg(debug_assertions)]
    {
        // Debug mode: read from filesystem
        let template_dir = Path::new(TEMPLATES_PATH).join(template_name);
        if !template_dir.exists() {
            anyhow::bail!("Template '{}' not found", template_name);
        }

        let metadata_path = template_dir.join("template.toml");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)?;
            let metadata: TemplateMetadata = toml::from_str(&content)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }
}

pub fn setup_handlebars() -> Handlebars<'static> {
    let handlebars = Handlebars::new();

    // Handlebars 6.x has built-in comparison helpers: eq, ne, gt, gte, lt, lte
    // and logical helpers: and, or, not - no need to register custom ones

    handlebars
}

pub fn load_template_metadata_from_path(
    template_path: &Path,
) -> anyhow::Result<Option<TemplateMetadata>> {
    let metadata_path = template_path.join("template.toml");
    if metadata_path.exists() {
        let content = fs::read_to_string(&metadata_path)?;
        let metadata: TemplateMetadata = toml::from_str(&content)?;
        Ok(Some(metadata))
    } else {
        Ok(None)
    }
}

pub fn process_template_directory(
    template_name: &str,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
) -> anyhow::Result<()> {
    #[cfg(not(debug_assertions))]
    {
        // Release mode: use embedded templates
        if let Some(template_dir) = TEMPLATES_DIR.get_dir(template_name) {
            process_embedded_template_directory(
                template_dir,
                output_dir,
                handlebars,
                data,
                metadata,
                "",
            )
        } else {
            anyhow::bail!("Template '{}' not found", template_name);
        }
    }

    #[cfg(debug_assertions)]
    {
        // Debug mode: use filesystem templates
        let template_dir = Path::new(TEMPLATES_PATH).join(template_name);
        if !template_dir.exists() {
            anyhow::bail!("Template '{}' not found", template_name);
        }

        process_filesystem_directory_recursive(
            &template_dir,
            output_dir,
            handlebars,
            data,
            metadata,
            "",
        )
    }
}

#[cfg(not(debug_assertions))]
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

        // Skip template.toml configuration files - they should not be copied
        if file_name == "template.toml" {
            continue;
        }

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
            if data
                .get("minimal")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                // Only apply full_mode_only exclusions at the template root level
                if relative_path.is_empty()
                    && metadata
                        .file_conventions
                        .full_mode_only
                        .contains(&subdir_name.to_string())
                {
                    continue;
                }
            }
        }

        // Template the directory name if needed
        let output_dirname = handlebars.render_template(subdir_name, data)?;
        let output_subdir = output_dir.join(&output_dirname);

        process_embedded_template_directory(
            subdir,
            &output_subdir,
            handlebars,
            data,
            metadata,
            &current_relative_path,
        )?;
    }

    Ok(())
}

fn get_file_config<'a>(
    metadata: Option<&'a TemplateMetadata>,
    relative_path: &str,
) -> Option<&'a FileConfig> {
    metadata?.files.iter().find(|f| f.path == relative_path)
}

fn should_skip_file(
    metadata: Option<&TemplateMetadata>,
    relative_path: &str,
    data: &serde_json::Value,
) -> bool {
    !should_include_file(metadata, relative_path, data)
}

fn should_include_file(
    metadata: Option<&TemplateMetadata>,
    relative_path: &str,
    data: &serde_json::Value,
) -> bool {
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

fn should_include_by_convention(
    conventions: &FileConventions,
    relative_path: &str,
    data: &serde_json::Value,
) -> bool {
    // Check if file is in always-included directories
    for always_dir in &conventions.always_include {
        if relative_path.starts_with(always_dir) {
            return true;
        }
    }

    // Check if file should be excluded in minimal mode
    let is_minimal = data
        .get("minimal")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if is_minimal {
        for full_only_dir in &conventions.full_mode_only {
            // Only exclude if we're at the root level (no parent directories)
            if relative_path == *full_only_dir {
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
        let parts: Vec<String> = condition
            .split("&&")
            .map(|part| convert_condition_to_handlebars(part.trim()))
            .collect();
        return format!("(and {})", parts.join(" "));
    }

    // Handle OR conditions
    if condition.contains("||") {
        let parts: Vec<String> = condition
            .split("||")
            .map(|part| convert_condition_to_handlebars(part.trim()))
            .collect();
        return format!("(or {})", parts.join(" "));
    }

    // Handle single conditions
    convert_single_condition_to_handlebars(condition)
}

fn convert_single_condition_to_handlebars(condition: &str) -> String {
    // Handle string equality: "integration == 'value'" -> "(eq integration 'value')"
    if let Some(captures) = regex::Regex::new(r"(\w+)\s*==\s*'([^']+)'")
        .unwrap()
        .captures(condition)
    {
        let var_name = captures.get(1).unwrap().as_str();
        let value = captures.get(2).unwrap().as_str();
        return format!("(eq {} '{}')", var_name, value);
    }

    // Handle boolean checks: "minimal == false" -> "(eq minimal false)"
    if let Some(captures) = regex::Regex::new(r"(\w+)\s*==\s*(true|false)")
        .unwrap()
        .captures(condition)
    {
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

pub fn process_filesystem_template_directory(
    template_path: &Path,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
) -> anyhow::Result<()> {
    process_filesystem_directory_recursive(
        template_path,
        output_dir,
        handlebars,
        data,
        metadata,
        "",
    )
}

/// Project characteristics detected from the target directory
#[derive(Debug, Default)]
pub struct ProjectCharacteristics {
    pub detected_xml_lib: bool,
    pub detected_library_target: Option<String>,
    pub cmake_version: Option<String>,
    pub sanitizer_mismatch_risk: bool,
    pub project_has_build_dir: bool,
}

/// Detect project characteristics from the target directory
pub fn detect_project_characteristics(output_dir: &Path) -> anyhow::Result<ProjectCharacteristics> {
    let mut characteristics = ProjectCharacteristics::default();
    
    // The output directory is typically the fuzz directory, so we need to go up one level
    // to analyze the parent project
    let project_root = output_dir.parent().unwrap_or(output_dir);
    
    // Detect XML parsing libraries
    characteristics.detected_xml_lib = has_xml_parsing_headers(project_root);
    
    // Detect CMake version and library targets
    if let Ok(cmake_info) = detect_cmake_info(project_root) {
        characteristics.cmake_version = cmake_info.version;
        characteristics.detected_library_target = cmake_info.primary_library_target;
        characteristics.project_has_build_dir = cmake_info.has_build_dir;
    }
    
    // Determine sanitizer mismatch risk
    characteristics.sanitizer_mismatch_risk = has_sanitizer_mismatch_risk(project_root);
    
    Ok(characteristics)
}

/// Check if the project uses XML parsing libraries
fn has_xml_parsing_headers(project_root: &Path) -> bool {
    // Look for common XML parsing headers and patterns
    let include_dirs = [
        project_root.join("include"),
        project_root.join("src"), 
        project_root.join("lib"),
    ];
    
    for include_dir in &include_dirs {
        if include_dir.exists() {
            // Check for common XML parsing library indicators
            if has_files_matching(include_dir, &[
                "parse.hpp", "parse.h",
                "xml.hpp", "xml.h", 
                "adm/parse.hpp",
                "rapidxml",
                "tinyxml",
                "pugixml",
                "libxml",
            ]) {
                return true;
            }
            
            // Check for XML-related directory structures
            if has_directories_matching(include_dir, &["adm", "xml", "rapidxml"]) {
                return true;
            }
        }
    }
    
    // Check for XML-related dependencies in CMakeLists.txt
    if let Ok(cmake_content) = fs::read_to_string(project_root.join("CMakeLists.txt")) {
        if cmake_content.contains("xml") || cmake_content.contains("XML") || 
           cmake_content.contains("adm") || cmake_content.contains("rapidxml") {
            return true;
        }
    }
    
    false
}

/// CMake project information
#[derive(Debug, Default)]
struct CMakeInfo {
    pub version: Option<String>,
    pub primary_library_target: Option<String>,
    pub has_build_dir: bool,
}

/// Detect CMake-related information
fn detect_cmake_info(project_root: &Path) -> anyhow::Result<CMakeInfo> {
    let mut info = CMakeInfo::default();
    
    // Check if there's a CMakeLists.txt
    let cmake_file = project_root.join("CMakeLists.txt");
    if !cmake_file.exists() {
        return Ok(info);
    }
    
    // Check for existing build directory
    info.has_build_dir = project_root.join("build").exists();
    
    // Read CMakeLists.txt to extract information
    let cmake_content = fs::read_to_string(&cmake_file)?;
    
    // Extract CMake version requirement
    if let Some(version_match) = regex::Regex::new(r"cmake_minimum_required\s*\(\s*VERSION\s+([0-9.]+)")
        .unwrap()
        .captures(&cmake_content) 
    {
        info.version = Some(version_match.get(1).unwrap().as_str().to_string());
    }
    
    // Try to detect the main library target
    info.primary_library_target = detect_library_target_from_cmake(&cmake_content, project_root);
    
    Ok(info)
}

/// Detect the primary library target from CMakeLists.txt content
fn detect_library_target_from_cmake(cmake_content: &str, project_root: &Path) -> Option<String> {
    // Look for add_library commands
    let library_regex = regex::Regex::new(r"add_library\s*\(\s*([^\s)]+)").unwrap();
    let mut library_targets = Vec::new();
    
    for capture in library_regex.captures_iter(cmake_content) {
        let target_name = capture.get(1).unwrap().as_str().to_string();
        // Skip if it looks like a variable reference
        if !target_name.starts_with('$') && !target_name.starts_with('{') {
            library_targets.push(target_name);
        }
    }
    
    // If we found library targets, try to pick the best one
    if !library_targets.is_empty() {
        // Get the project name from the directory or CMakeLists.txt
        let project_name = get_project_name_from_cmake(cmake_content, project_root);
        
        // Prefer targets that match the project name
        if let Some(project_name) = &project_name {
            for target in &library_targets {
                if target == project_name || 
                   target == &format!("lib{}", project_name) ||
                   target == &format!("{}_lib", project_name) {
                    return Some(target.clone());
                }
            }
        }
        
        // If no perfect match, return the first library target
        Some(library_targets[0].clone())
    } else {
        None
    }
}

/// Extract project name from CMakeLists.txt or directory name
fn get_project_name_from_cmake(cmake_content: &str, project_root: &Path) -> Option<String> {
    // Try to extract from project() command
    if let Some(project_match) = regex::Regex::new(r"project\s*\(\s*([^\s)]+)")
        .unwrap()
        .captures(cmake_content) 
    {
        let project_name = project_match.get(1).unwrap().as_str();
        if !project_name.starts_with('$') {
            return Some(project_name.to_string());
        }
    }
    
    // Fall back to directory name
    project_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

/// Check if there's a risk of sanitizer mismatch
fn has_sanitizer_mismatch_risk(project_root: &Path) -> bool {
    // If there's an existing build directory but it's not a sanitizer build,
    // there's a risk of mismatch
    let build_dir = project_root.join("build");
    if build_dir.exists() {
        // Check CMakeCache.txt for sanitizer flags
        if let Ok(cache_content) = fs::read_to_string(build_dir.join("CMakeCache.txt")) {
            // If no sanitizer flags are found in the existing build, there's a mismatch risk
            return !cache_content.contains("fsanitize");
        }
    }
    false
}

/// Check if directory contains files matching any of the patterns
fn has_files_matching(dir: &Path, patterns: &[&str]) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                for pattern in patterns {
                    if file_name.contains(pattern) {
                        return true;
                    }
                }
            }
            
            // Recursively check subdirectories (limited depth)
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if has_files_matching(&entry.path(), patterns) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if directory contains subdirectories matching any of the patterns
fn has_directories_matching(dir: &Path, patterns: &[&str]) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(dir_name) = entry.file_name().to_str() {
                    for pattern in patterns {
                        if dir_name.contains(pattern) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Add project characteristics to template data
pub fn enrich_template_data(
    mut data: serde_json::Value,
    output_dir: &Path,
    template_name: &str,
    template_path: Option<&Path>,
) -> anyhow::Result<serde_json::Value> {
    let characteristics = detect_project_characteristics(output_dir)?;
    
    // Add characteristics to template data
    if let Some(data_obj) = data.as_object_mut() {
        data_obj.insert("detected_xml_lib".to_string(), 
                       serde_json::Value::Bool(characteristics.detected_xml_lib));
        
        if let Some(target) = characteristics.detected_library_target {
            data_obj.insert("detected_library_target".to_string(), 
                           serde_json::Value::String(target));
        }
        
        if let Some(version) = characteristics.cmake_version {
            data_obj.insert("cmake_version".to_string(), 
                           serde_json::Value::String(version));
        }
        
        data_obj.insert("sanitizer_mismatch_risk".to_string(), 
                       serde_json::Value::Bool(characteristics.sanitizer_mismatch_risk));
        
        data_obj.insert("project_has_build_dir".to_string(), 
                       serde_json::Value::Bool(characteristics.project_has_build_dir));

        // Run project analysis if analysis script exists
        if let Ok(analysis_data) = run_project_analysis(output_dir, template_name, template_path) {
            // Merge analysis results into template data
            for (key, value) in analysis_data.as_object().unwrap_or(&serde_json::Map::new()) {
                data_obj.insert(key.clone(), value.clone());
            }
        }
    }
    
    Ok(data)
}

/// Run project analysis script if it exists in the template
fn run_project_analysis(
    project_dir: &Path,
    template_name: &str,
    template_path: Option<&Path>,
) -> anyhow::Result<serde_json::Value> {
    // Find the analysis script
    let script_path = if let Some(path) = template_path {
        // Remote template - look in filesystem
        path.join("analyze_project.sh")
    } else {
        // Embedded template - extract to temp and run
        #[cfg(debug_assertions)]
        {
            Path::new(TEMPLATES_PATH).join(template_name).join("analyze_project.sh")
        }
        #[cfg(not(debug_assertions))]
        {
            // For release builds, we'd need to extract the script first
            return Err(anyhow::anyhow!("Project analysis not supported in release builds yet"));
        }
    };

    if !script_path.exists() {
        return Err(anyhow::anyhow!("No analysis script found"));
    }

    // Create temp file for JSON output
    let temp_json = tempfile::NamedTempFile::new()?;
    let temp_path = temp_json.path();

    // Run the analysis script
    let output = Command::new("bash")
        .arg(&script_path)
        .arg(project_dir)
        .arg(temp_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Analysis script failed: {}", stderr));
    }

    // Read and parse the JSON result
    let json_content = fs::read_to_string(temp_path)?;
    let analysis_result: serde_json::Value = serde_json::from_str(&json_content)?;

    Ok(analysis_result)
}

fn process_filesystem_directory_recursive(
    template_dir: &Path,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
    relative_path: &str,
) -> anyhow::Result<()> {
    // Create the output directory
    fs::create_dir_all(output_dir)?;

    // Process all entries in the directory
    for entry in fs::read_dir(template_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        let current_relative_path = if relative_path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", relative_path, file_name)
        };

        if file_type.is_dir() {
            // Check directory inclusion rules
            if let Some(metadata) = metadata {
                // Check if this directory should be excluded in minimal mode
                if data
                    .get("minimal")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    // Only apply full_mode_only exclusions at the template root level
                    if relative_path.is_empty()
                        && metadata
                            .file_conventions
                            .full_mode_only
                            .contains(&file_name)
                    {
                        continue;
                    }
                }
            }

            // Template the directory name if needed
            let output_dirname = handlebars.render_template(&file_name, data)?;
            let output_subdir = output_dir.join(&output_dirname);

            process_filesystem_directory_recursive(
                &entry.path(),
                &output_subdir,
                handlebars,
                data,
                metadata,
                &current_relative_path,
            )?;
        } else if file_type.is_file() {
            // Skip template.toml configuration files - they should not be copied
            if file_name == "template.toml" {
                continue;
            }

            // Check if this file should be included based on conditions
            if should_skip_file(metadata, &current_relative_path, data) {
                continue;
            }

            // Check if this file should be templated
            let file_config = get_file_config(metadata, &current_relative_path);
            let should_template = file_config.map_or(true, |fc| fc.template);

            // Template the filename if needed
            let output_filename = if should_template {
                handlebars.render_template(&file_name, data)?
            } else {
                file_name.clone()
            };

            let output_path = output_dir.join(&output_filename);

            // Process file content
            let content = fs::read(&entry.path())?;

            // Try to process as UTF-8 text
            if let Ok(text_content) = String::from_utf8(content.clone()) {
                if should_template {
                    let rendered = handlebars.render_template(&text_content, data)?;
                    // Skip empty files (allows Handlebars conditionals to hide entire files)
                    if rendered.trim().is_empty() {
                        continue;
                    }
                    fs::write(&output_path, rendered)?;
                } else {
                    fs::write(&output_path, text_content)?;
                }
            } else {
                // Binary file - write as-is
                fs::write(&output_path, content)?;
            }

            // Set executable permissions if needed
            if file_config.map_or(false, |fc| fc.executable) {
                set_executable(&output_path)?;
            }
        }
    }

    Ok(())
}
