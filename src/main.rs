use std::{fs, path::{Path, PathBuf}, collections::HashMap, process::Command};
use inquire::{Text, Select};
use handlebars::Handlebars;
use serde_json::json;
use clap::Parser;
use serde::{Deserialize, Serialize};
use reqwest;
use tempfile::TempDir;
use include_dir::{include_dir, Dir};

// Embed the templates directory at compile time
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/templates");

fn get_available_templates() -> anyhow::Result<Vec<String>> {
    let mut templates = Vec::new();
    
    for entry in TEMPLATES_DIR.dirs() {
        templates.push(entry.path().file_name().unwrap().to_str().unwrap().to_string());
    }
    
    templates.sort();
    Ok(templates)
}

#[derive(Debug, Deserialize, Serialize)]
struct FuzzerOption {
    name: String,
    display_name: String,
    description: String,
    requires: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FuzzerConfig {
    supported: Vec<String>,
    default: String,
    options: Vec<FuzzerOption>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrationOption {
    name: String,
    description: String,
    files: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntegrationConfig {
    supported: Vec<String>,
    default: String,
    options: Vec<IntegrationOption>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TemplateMetadata {
    template: TemplateInfo,
    variables: HashMap<String, VariableConfig>,
    #[serde(default)]
    files: Vec<FileConfig>,
    #[serde(default)]
    directories: Vec<DirectoryConfig>,
    #[serde(default)]
    hooks: HookConfig,
    #[serde(default)]
    fuzzers: Option<FuzzerConfig>,
    #[serde(default)]
    integrations: Option<IntegrationConfig>,
    #[serde(default)]
    file_conventions: FileConventions,
}

#[derive(Debug, Deserialize, Serialize)]
struct TemplateInfo {
    name: String,
    description: String,
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct VariableConfig {
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileConfig {
    path: String,
    #[serde(default)]
    executable: bool,
    #[serde(default = "default_true")]
    template: bool,
    #[serde(default)]
    condition: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DirectoryConfig {
    path: String,
    #[serde(default)]
    create_empty: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct HookConfig {
    #[serde(default)]
    post_generate: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct FileConventions {
    #[serde(default)]
    always_include: Vec<String>,
    #[serde(default)]
    full_mode_only: Vec<String>,
    #[serde(default)]
    template_extensions: Vec<String>,
    #[serde(default)]
    executable_extensions: Vec<String>,
    #[serde(default)]
    no_template_extensions: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
enum TemplateSource {
    Local(String),           // Local template name like "c" or "rust"
    GitHub { org: String, repo: String, path: Option<String> }, // github:org/repo or @org/repo
    GitHubFull(String),      // Full github URL like github:forallsecure/c-fuzzme
}

impl TemplateSource {
    fn parse(input: &str) -> anyhow::Result<Self> {
        if input.starts_with("github:") {
            let github_spec = input.strip_prefix("github:").unwrap();
            if github_spec.contains('/') {
                let parts: Vec<&str> = github_spec.split('/').collect();
                if parts.len() >= 2 {
                    Ok(TemplateSource::GitHub {
                        org: parts[0].to_string(),
                        repo: parts[1].to_string(),
                        path: if parts.len() > 2 { Some(parts[2..].join("/")) } else { None },
                    })
                } else {
                    anyhow::bail!("Invalid GitHub template format. Expected: github:org/repo")
                }
            } else {
                Ok(TemplateSource::GitHubFull(github_spec.to_string()))
            }
        } else if input.starts_with('@') {
            let org_repo = input.strip_prefix('@').unwrap();
            let parts: Vec<&str> = org_repo.split('/').collect();
            if parts.len() >= 2 {
                Ok(TemplateSource::GitHub {
                    org: parts[0].to_string(),
                    repo: parts[1].to_string(),
                    path: if parts.len() > 2 { Some(parts[2..].join("/")) } else { None },
                })
            } else {
                anyhow::bail!("Invalid @ template format. Expected: @org/repo")
            }
        } else {
            Ok(TemplateSource::Local(input.to_string()))
        }
    }
}

async fn fetch_github_template(source: &TemplateSource) -> anyhow::Result<TempDir> {
    let (org, repo, path_opt) = match source {
        TemplateSource::GitHub { org, repo, path } => {
            (org.as_str(), repo.as_str(), path.clone())
        }
        TemplateSource::GitHubFull(spec) => {
            let parts: Vec<&str> = spec.split('/').collect();
            if parts.len() >= 2 {
                let path_opt = if parts.len() > 2 { 
                    Some(parts[2..].join("/"))
                } else { 
                    None 
                };
                (parts[0], parts[1], path_opt)
            } else {
                anyhow::bail!("Invalid GitHub specification: {}", spec)
            }
        }
        _ => anyhow::bail!("Not a GitHub template source"),
    };
    
    let path = path_opt.as_deref();

    println!("Fetching template from GitHub: {}/{}", org, repo);
    
    // Download the repository as a ZIP file
    let download_url = format!("https://api.github.com/repos/{}/{}/zipball", org, repo);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", "fuzz-init")
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to download template: HTTP {}", response.status());
    }
    
    let bytes = response.bytes().await?;
    
    // Create temporary directory and extract ZIP
    let temp_dir = tempfile::tempdir()?;
    let zip_path = temp_dir.path().join("template.zip");
    fs::write(&zip_path, &bytes)?;
    
    // Extract ZIP file
    let file = fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // Find the root directory in the ZIP (GitHub creates a folder like "repo-branch")
    let mut root_dir_name = None;
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let file_path = file.name();
        if let Some(first_component) = file_path.split('/').next() {
            if root_dir_name.is_none() {
                root_dir_name = Some(first_component.to_string());
            }
            break;
        }
    }
    
    let root_dir = root_dir_name.ok_or_else(|| anyhow::anyhow!("Could not find root directory in ZIP"))?;
    
    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.name();
        
        // Skip directories
        if file_path.ends_with('/') {
            continue;
        }
        
        // Remove the root directory prefix
        let relative_path = file_path.strip_prefix(&format!("{}/", root_dir))
            .unwrap_or(file_path);
        
        // If a specific path is requested, only extract files under that path
        if let Some(requested_path) = path {
            if !relative_path.starts_with(requested_path) {
                continue;
            }
            // Remove the requested path prefix as well
            let final_path = relative_path.strip_prefix(&format!("{}/", requested_path))
                .unwrap_or(relative_path);
            
            let out_path = temp_dir.path().join(final_path);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        } else {
            let out_path = temp_dir.path().join(relative_path);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }
    
    // Clean up the ZIP file
    fs::remove_file(&zip_path)?;
    
    Ok(temp_dir)
}

fn load_template_metadata(template_name: &str) -> anyhow::Result<Option<TemplateMetadata>> {
    if let Some(template_dir) = TEMPLATES_DIR.get_dir(template_name) {
        if let Some(metadata_file) = template_dir.get_file("template.toml") {
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

fn get_file_config<'a>(metadata: Option<&'a TemplateMetadata>, relative_path: &str) -> Option<&'a FileConfig> {
    metadata?.files.iter().find(|f| f.path == relative_path)
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

fn should_skip_file(metadata: Option<&TemplateMetadata>, relative_path: &str, data: &serde_json::Value) -> bool {
    !should_include_file(metadata, relative_path, data)
}

fn should_include_by_convention(conventions: &FileConventions, relative_path: &str, data: &serde_json::Value) -> bool {
    // Check if file is in always-included directories
    for always_dir in &conventions.always_include {
        if relative_path.starts_with(always_dir) {
            // Files in always-included directories are included by default
            // but still subject to minimal mode rules for tutorial files
            if is_tutorial_file(relative_path) {
                if let Some(minimal) = data.get("minimal") {
                    if let Some(minimal_bool) = minimal.as_bool() {
                        return !minimal_bool; // Exclude tutorial files in minimal mode
                    }
                }
            }
            return true;
        }
    }
    
    // Check directory-based rules for full-mode-only directories
    for full_mode_dir in &conventions.full_mode_only {
        if relative_path.starts_with(full_mode_dir) {
            // Only include if not in minimal mode
            if let Some(minimal) = data.get("minimal") {
                if let Some(minimal_bool) = minimal.as_bool() {
                    return !minimal_bool;
                }
            }
            return true; // Default to include if minimal not specified
        }
    }
    
    true // Include by default
}

fn is_tutorial_file(relative_path: &str) -> bool {
    let filename = Path::new(relative_path).file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    // Tutorial/documentation files that should be excluded in minimal mode
    matches!(filename, "Dockerfile" | "Mayhemfile" | "README.md")
}

fn evaluate_condition(condition: &str, data: &serde_json::Value) -> bool {
    // Handle AND conditions first (higher precedence)
    if condition.contains("&&") {
        return condition.split("&&")
            .map(|part| part.trim())
            .all(|part| evaluate_condition_with_or(part, data));
    }
    
    // Handle OR conditions
    evaluate_condition_with_or(condition, data)
}

fn evaluate_condition_with_or(condition: &str, data: &serde_json::Value) -> bool {
    if condition.contains("||") {
        return condition.split("||")
            .map(|part| part.trim())
            .any(|part| evaluate_single_condition(part, data));
    }
    
    // Single condition
    evaluate_single_condition(condition, data)
}

fn evaluate_single_condition(condition: &str, data: &serde_json::Value) -> bool {
    // Handle string equality: "integration == 'value'"
    if let Some(captures) = regex::Regex::new(r"(\w+)\s*==\s*'([^']+)'").unwrap().captures(condition) {
        let variable = captures.get(1).unwrap().as_str();
        let expected_value = captures.get(2).unwrap().as_str();
        
        if let Some(actual_value) = data.get(variable) {
            if let Some(actual_str) = actual_value.as_str() {
                return actual_str == expected_value;
            }
        }
        return false;
    }
    
    // Handle boolean equality: "minimal == false" or "minimal == true"
    if let Some(captures) = regex::Regex::new(r"(\w+)\s*==\s*(true|false)").unwrap().captures(condition) {
        let variable = captures.get(1).unwrap().as_str();
        let expected_bool = captures.get(2).unwrap().as_str() == "true";
        
        if let Some(actual_value) = data.get(variable) {
            if let Some(actual_bool) = actual_value.as_bool() {
                return actual_bool == expected_bool;
            }
        }
        return false;
    }
    
    false // Unknown condition format, don't include
}

fn should_template_file(metadata: Option<&TemplateMetadata>, relative_path: &str, file_path: &Path) -> bool {
    // First check explicit file configuration
    if let Some(config) = get_file_config(metadata, relative_path) {
        return config.template;
    }
    
    // Apply convention-based rules
    if let Some(metadata) = metadata {
        return should_template_by_convention(&metadata.file_conventions, file_path);
    }
    
    // Default behavior: template text files
    is_text_file(file_path)
}

fn should_template_by_convention(conventions: &FileConventions, file_path: &Path) -> bool {
    if let Some(extension) = file_path.extension() {
        let ext = format!(".{}", extension.to_string_lossy());
        
        // Check if explicitly marked as no-template
        if conventions.no_template_extensions.contains(&ext) {
            return false;
        }
        
        // Check if explicitly marked as template
        if !conventions.template_extensions.is_empty() && conventions.template_extensions.contains(&ext) {
            return true;
        }
    }
    
    // Fall back to default text file detection
    is_text_file(file_path)
}

fn should_be_executable(metadata: Option<&TemplateMetadata>, relative_path: &str, file_path: &Path) -> anyhow::Result<bool> {
    // First check explicit file configuration
    if let Some(config) = get_file_config(metadata, relative_path) {
        return Ok(config.executable);
    }
    
    // Apply convention-based rules
    if let Some(metadata) = metadata {
        if should_be_executable_by_convention(&metadata.file_conventions, file_path) {
            return Ok(true);
        }
    }
    
    // Default behavior: check existing permissions
    is_executable(file_path)
}

fn should_be_executable_by_convention(conventions: &FileConventions, file_path: &Path) -> bool {
    if let Some(extension) = file_path.extension() {
        let ext = format!(".{}", extension.to_string_lossy());
        return conventions.executable_extensions.contains(&ext);
    }
    false
}

fn process_template_directory(
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
                handlebars.render_template(utf8_content, data)?
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

fn process_template_directory_impl(
    template_dir: &Path,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
    root_template_dir: &Path,
) -> anyhow::Result<()> {
    // Create the output directory
    fs::create_dir_all(output_dir)?;
    
    // Process all entries in the template directory
    for entry in fs::read_dir(template_dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();
        
        // Skip template.toml metadata file
        if entry_name == "template.toml" {
            continue;
        }
        
        // Apply templating to filename if it contains handlebars syntax
        let filename_str = entry_name.to_string_lossy();
        let templated_filename = if filename_str.contains("{{") && filename_str.contains("}}") {
            handlebars.render_template(&filename_str, data)?
        } else {
            filename_str.to_string()
        };
        let output_path = output_dir.join(&templated_filename);
        
        if entry_path.is_dir() {
            // Recursively process subdirectories
            process_template_directory_impl(&entry_path, &output_path, handlebars, data, metadata, root_template_dir)?;
        } else {
            // Calculate relative path from root template directory
            let relative_path = entry_path.strip_prefix(root_template_dir)
                .map_err(|_| anyhow::anyhow!("Failed to calculate relative path"))?
                .to_string_lossy();
            
            // Check if file should be included based on condition
            if !should_include_file(metadata, &relative_path, data) {
                continue; // Skip this file
            }
            
            // Process files based on metadata configuration
            if should_template_file(metadata, &relative_path, &entry_path) {
                // Apply templating to text files
                let content = fs::read_to_string(&entry_path)?;
                let rendered = handlebars.render_template(&content, data)?;
                fs::write(&output_path, rendered)?;
            } else {
                // Copy files directly without templating
                fs::copy(&entry_path, &output_path)?;
            }
            
            // Set executable permissions based on metadata
            if should_be_executable(metadata, &relative_path, &entry_path)? {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut permissions = fs::metadata(&output_path)?.permissions();
                    permissions.set_mode(permissions.mode() | 0o755);
                    fs::set_permissions(&output_path, permissions)?;
                }
            }
        }
    }
    
    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "txt" | "md" | "rs" | "c" | "cpp" | "h" | "hpp" | "py" | "js" | "ts" | "json" | "toml" | "yaml" | "yml" | "sh" | "dockerfile" | "makefile")
    } else {
        // Files without extensions - check some common names
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy().to_lowercase();
            matches!(name_str.as_str(), "dockerfile" | "makefile" | "mayhemfile" | "readme")
        } else {
            false
        }
    }
}

fn is_executable(path: &Path) -> anyhow::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();
        Ok(permissions.mode() & 0o111 != 0)
    }
    
    #[cfg(not(unix))]
    {
        // On non-Unix systems, check file extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            Ok(matches!(ext.as_str(), "sh" | "exe" | "bat" | "cmd"))
        } else {
            Ok(false)
        }
    }
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

#[derive(Parser)]
#[command(name = "fuzz-init")]
#[command(about = "Scaffold fuzz harnesses with Mayhem for various languages")]
#[command(version)]

struct Args {

    /// Positional argument for project name. 
    #[arg()]
    project_name_pos: Option<String>,

    /// Named flag alternative to positional.
    #[arg(long)]
    project: Option<String>,
    
    /// Programming language for the template (c, cpp, python, rust)
    #[arg(long)]
    language: Option<String>,
    
    /// Build system integration type (standalone, makefile, cmake)
    #[arg(long)]
    integration: Option<String>,
    
    /// Fuzzer type to use as default (afl, libfuzzer, honggfuzz, standalone)
    #[arg(long)]
    fuzzer: Option<String>,
    
    /// Template to use (github:org/repo, or @org/repo)
    #[arg(long)]
    template: Option<String>,
    
    /// Generate minimal template (fuzz directory only) instead of full tutorial
    #[arg(long)]
    minimal: bool,
    
    /// Test template functionality by generating and building with all fuzzer options
    #[arg(long)]
    test: bool,
}

fn get_project_name(args: &Args) -> anyhow::Result<String> {
    match args.project.as_ref().or(args.project_name_pos.as_ref()) {
        Some(name) => Ok(name.clone()),
        None => Ok(Text::new("Project name:").prompt()?),
    }
}

fn determine_template_source(args: &Args, available_templates: &[String]) -> anyhow::Result<TemplateSource> {
    match (&args.language, &args.template) {
        // Language specified - use local template
        (Some(language), None) => {
            if !available_templates.contains(language) {
                anyhow::bail!("Invalid language '{}'. Available languages: {}", language, available_templates.join(", "));
            }
            Ok(TemplateSource::Local(language.clone()))
        }
        // Template specified (remote or local for backward compatibility)
        (None, Some(template_str)) => {
            if template_str.starts_with("github:") || template_str.starts_with('@') {
                // Remote template
                TemplateSource::parse(template_str)
            } else {
                // Local template (backward compatibility)
                if !available_templates.contains(template_str) {
                    anyhow::bail!("Invalid template '{}'. Available templates: {}", template_str, available_templates.join(", "));
                }
                Ok(TemplateSource::Local(template_str.clone()))
            }
        }
        // Both specified - error (conflicting options)
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both --language and --template. Use --language for local templates, --template for remote templates.");
        }
        // Neither specified - prompt for language
        (None, None) => {
            let selected = Select::new("Choose a language", available_templates.to_vec()).prompt()?;
            Ok(TemplateSource::Local(selected))
        }
    }
}

async fn get_template_name(template_source: &TemplateSource, available_templates: &[String]) -> anyhow::Result<(String, Option<TempDir>)> {
    match template_source {
        TemplateSource::Local(name) => {
            if !available_templates.contains(name) {
                anyhow::bail!("Invalid template name. Available templates: {}", available_templates.join(", "));
            }
            Ok((name.clone(), None))
        }
        _ => {
            // For remote templates, we still need to fetch them to temp directory
            // This keeps the existing GitHub template functionality
            let temp_dir = fetch_github_template(template_source).await?;
            let path = temp_dir.path().to_path_buf();
            
            // For remote templates, we'll use the old filesystem-based approach
            // until we implement embedding for remote templates too
            anyhow::bail!("Remote templates not yet supported with embedded template system. Use local templates only.")
        }
    }
}

fn select_fuzzer(args: &Args, metadata: Option<&TemplateMetadata>) -> anyhow::Result<String> {
    if let Some(fuzzer) = &args.fuzzer {
        // Validate fuzzer type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(fuzzer_config) = &metadata.fuzzers {
                if !fuzzer_config.supported.contains(fuzzer) {
                    anyhow::bail!("Invalid fuzzer type '{}'. Supported: {}", 
                        fuzzer, fuzzer_config.supported.join(", "));
                }
            }
        }
        Ok(fuzzer.clone())
    } else if let Some(metadata) = metadata {
        if let Some(fuzzer_config) = &metadata.fuzzers {
            if fuzzer_config.options.len() > 1 {
                // Build display options from metadata
                let fuzzer_options: Vec<String> = fuzzer_config.options.iter()
                    .map(|opt| format!("{} - {} ({})", opt.display_name, opt.description, opt.requires))
                    .collect();
                
                // Find the default option index
                let default_index = fuzzer_config.options.iter()
                    .position(|opt| opt.name == fuzzer_config.default)
                    .unwrap_or(0);
                
                let selection = Select::new("Choose default fuzzer type:", fuzzer_options)
                    .with_starting_cursor(default_index)
                    .prompt()?;
                
                // Extract the fuzzer name from the selection
                let display_name = selection.split(" - ").next().unwrap();
                Ok(fuzzer_config.options.iter()
                    .find(|opt| opt.display_name == display_name)
                    .map(|opt| opt.name.clone())
                    .unwrap_or_else(|| fuzzer_config.default.clone()))
            } else {
                Ok(fuzzer_config.default.clone())
            }
        } else {
            Ok("standalone".to_string())
        }
    } else {
        Ok("standalone".to_string())
    }
}

fn select_integration(args: &Args, metadata: Option<&TemplateMetadata>) -> anyhow::Result<String> {
    if let Some(integration) = &args.integration {
        // Validate integration type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(integration_config) = &metadata.integrations {
                if !integration_config.supported.contains(integration) {
                    anyhow::bail!("Invalid integration type '{}'. Supported: {}", 
                        integration, integration_config.supported.join(", "));
                }
            }
        }
        Ok(integration.clone())
    } else if let Some(metadata) = metadata {
        if let Some(integration_config) = &metadata.integrations {
            if integration_config.options.len() > 1 {
                // Build display options from metadata
                let integration_options: Vec<String> = integration_config.options.iter()
                    .map(|opt| format!("{} - {}", opt.name, opt.description))
                    .collect();
                
                // Find the default option index
                let default_index = integration_config.options.iter()
                    .position(|opt| opt.name == integration_config.default)
                    .unwrap_or(0);
                
                let selection = Select::new("Choose integration type:", integration_options)
                    .with_starting_cursor(default_index)
                    .prompt()?;
                
                // Extract the integration name from the selection
                let integration_name = selection.split(" - ").next().unwrap();
                Ok(integration_name.to_string())
            } else {
                Ok(integration_config.default.clone())
            }
        } else {
            Ok("standalone".to_string())
        }
    } else {
        Ok("standalone".to_string())
    }
}

fn determine_minimal_mode(args: &Args, template_source: &TemplateSource) -> bool {
    if matches!(template_source, TemplateSource::Local(_)) {
        args.minimal
    } else {
        // Remote templates are always "full" (whatever they contain)
        false
    }
}

fn get_template_display_name(template_source: &TemplateSource) -> String {
    match template_source {
        TemplateSource::Local(name) => name.clone(),
        TemplateSource::GitHub { org, repo, .. } => format!("{}/{}", org, repo),
        TemplateSource::GitHubFull(spec) => spec.clone(),
    }
}

fn print_next_steps(project_name: &str, default_fuzzer: &str, integration_type: &str, minimal_mode: bool, metadata: Option<&TemplateMetadata>) {
    println!();
    println!("🚀 Next steps:");
    println!("==============");
    
    // Step 1: Change directory
    println!("1. cd {}", project_name);
    
    // Generic instructions that work for all templates
    match integration_type {
        "cmake" => {
            println!("2. mkdir build && cd build");
            if default_fuzzer == "libfuzzer" {
                println!("3. CC=clang cmake ..");
            } else {
                println!("3. cmake ..");
            }
            println!("4. cmake --build . --target fuzz");
        }
        "make" => {
            println!("2. make fuzz             # Build the fuzzer");
        }
        "standalone" => {
            println!("2. cd fuzz");
            println!("3. ./build.sh           # Build with {} fuzzer", default_fuzzer);
        }
        _ => {
            // Use template metadata if available for unknown types
            if let Some(template_meta) = metadata {
                if let Some(integration_config) = &template_meta.integrations {
                    if let Some(integration_option) = integration_config.options.iter().find(|opt| opt.name == integration_type) {
                        println!("2. # {} - {}", integration_type, integration_option.description);
                        println!("3. # See INTEGRATION.md for detailed instructions");
                    } else {
                        println!("2. # Follow build instructions in INTEGRATION.md");
                    }
                } else {
                    println!("2. # Follow build instructions in INTEGRATION.md");
                }
            } else {
                println!("2. # Follow build instructions in INTEGRATION.md");
            }
        }
    }
    
    // Generic run instructions
    println!();
    println!("🔍 Run your fuzzer:");
    match integration_type {
        "cmake" => {
            println!("   cd fuzz/build && ./{}_{} ../testsuite/", project_name, default_fuzzer);
        }
        "make" | "standalone" => {
            println!("   cd fuzz && ./{}  testsuite/", format!("{}-{}", project_name, default_fuzzer));
        }
        _ => {
            println!("   # See fuzz/README.md for run instructions");
        }
    }
    
    // Documentation pointers
    println!();
    println!("📚 Documentation:");
    if minimal_mode {
        println!("   • fuzz/INTEGRATION.md  - Integration guide for existing projects");
        println!("   • fuzz/README.md       - Quick reference for fuzzing commands");
    } else {
        println!("   • TUTORIAL.md          - Complete fuzzing tutorial and examples");
        println!("   • fuzz/INTEGRATION.md  - Integration guide for existing projects");
        println!("   • fuzz/README.md       - Quick reference for fuzzing commands");
    }
    
    // Template-specific information from metadata
    if let Some(template_meta) = metadata {
        println!();
        println!("💡 Template info:");
        println!("   • Language: {}", template_meta.template.name);
        println!("   • Description: {}", template_meta.template.description);
        
        if let Some(fuzzer_config) = &template_meta.fuzzers {
            let other_fuzzers: Vec<&str> = fuzzer_config.supported.iter()
                .filter(|&f| f != default_fuzzer)
                .map(|s| s.as_str())
                .collect();
            if !other_fuzzers.is_empty() {
                println!("   • Other fuzzer types available: {}", other_fuzzers.join(", "));
            }
        }
    }
    
    println!();
    println!("Happy fuzzing! 🐛");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Handle test mode
    if args.test {
        return run_template_tests().await;
    }
    
    // Get available templates
    let available_templates = get_available_templates()?;
    if available_templates.is_empty() {
        anyhow::bail!("No templates found in src/templates directory");
    }
    
    // Get all necessary inputs
    let project_name = get_project_name(&args)?;
    let template_source = determine_template_source(&args, &available_templates)?;
    let (template_name, _temp_dir) = get_template_name(&template_source, &available_templates).await?;
    
    // Load template metadata
    let metadata = load_template_metadata(&template_name)?;
    
    // Get user selections
    let default_fuzzer = select_fuzzer(&args, metadata.as_ref())?;
    let integration_type = select_integration(&args, metadata.as_ref())?;
    let minimal_mode = determine_minimal_mode(&args, &template_source);
    
    // Create template data
    let mut handlebars = Handlebars::new();
    
    // Register the 'eq' helper for conditional templating
    handlebars.register_helper("eq", Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
        let param0 = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let param1 = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
        if param0 == param1 {
            out.write("true")?;
        }
        Ok(())
    }));
    
    let data = json!({ 
        "project_name": project_name,
        "target_name": project_name, // Use project name as target name by default
        "default_fuzzer": default_fuzzer,
        "integration": integration_type,
        "minimal": minimal_mode
    });
    
    // Generate project
    let out_path_string = format!("./{}", project_name);
    let out_path = Path::new(&out_path_string);
    process_template_directory(&template_name, out_path, &handlebars, &data, metadata.as_ref())?;
    
    // Success message with next steps
    println!("Project '{}' created with {} template!", project_name, template_name);
    
    print_next_steps(&project_name, &default_fuzzer, &integration_type, minimal_mode, metadata.as_ref());
    
    Ok(())
}

async fn run_template_tests() -> anyhow::Result<()> {
    println!("🧪 Running comprehensive template tests...\n");
    
    // Create test output directory
    let test_dir = Path::new("./template-tests");
    if test_dir.exists() {
        println!("🗑️  Removing existing test directory...");
        fs::remove_dir_all(test_dir)?;
    }
    fs::create_dir_all(test_dir)?;
    println!("📁 Created test directory: {}", test_dir.display());
    
    let available_templates = get_available_templates()?;
    let mut all_combinations = Vec::new();
    
    // Phase 1: Generate all template combinations
    println!("\n📦 Phase 1: Generating all template combinations...");
    println!("================================================");
    
    for template_name in &available_templates {
        println!("Processing template: {}", template_name);
        
        // Load template metadata to get all options
        let metadata = load_template_metadata(template_name)?;
        
        let combinations = generate_template_combinations(template_name, &metadata, test_dir).await?;
        all_combinations.extend(combinations);
        
        println!("  Generated {} combinations", all_combinations.len());
    }
    
    println!("\n✅ Generated {} total template combinations in {}", all_combinations.len(), test_dir.display());
    
    // Phase 2: Test all generated combinations
    println!("\n🔧 Phase 2: Testing builds for all combinations...");
    println!("================================================");
    
    let mut test_results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    
    for (combination_name, combination_path) in &all_combinations {
        println!("Testing: {}", combination_name);
        
        let success = test_combination_build(combination_path).await?;
        test_results.push((combination_name.clone(), success));
        
        if success {
            println!("  ✅ Build successful");
            passed += 1;
        } else {
            println!("  ❌ Build failed");
            failed += 1;
        }
    }
    
    // Print final summary
    println!("\n📊 Final Test Summary:");
    println!("====================");
    println!("Total combinations generated: {}", all_combinations.len());
    println!("Build tests passed: {}", passed);
    println!("Build tests failed: {}", failed);
    println!("Test output directory: {}", test_dir.display());
    
    if failed > 0 {
        println!("\n❌ Failed combinations:");
        for (combination_name, success) in &test_results {
            if !success {
                println!("   • {}", combination_name);
            }
        }
        println!("\n💡 You can inspect the generated projects in {} and manually debug build issues.", test_dir.display());
        anyhow::bail!("{} out of {} template combinations failed build tests", failed, all_combinations.len());
    }
    
    println!("\n🎉 All template combinations generated and tested successfully!");
    println!("💡 You can inspect the generated projects in {} before deleting for check-in.", test_dir.display());
    
    Ok(())
}

async fn generate_template_combinations(
    template_name: &str, 
    metadata: &Option<TemplateMetadata>, 
    test_dir: &Path
) -> anyhow::Result<Vec<(String, PathBuf)>> {
    let mut combinations = Vec::new();
    
    // Get all possible options from metadata
    let fuzzer_options = if let Some(ref metadata) = metadata {
        if let Some(ref fuzzer_config) = metadata.fuzzers {
            fuzzer_config.options.iter().map(|opt| opt.name.clone()).collect()
        } else {
            vec!["standalone".to_string()]
        }
    } else {
        vec!["standalone".to_string()]
    };
    
    let integration_options = if let Some(ref metadata) = metadata {
        if let Some(ref integration_config) = metadata.integrations {
            integration_config.options.iter().map(|opt| opt.name.clone()).collect()
        } else {
            vec!["standalone".to_string()]
        }
    } else {
        vec!["standalone".to_string()]
    };
    
    let minimal_options = vec![false, true];
    
    println!("  Fuzzer options: {}", fuzzer_options.join(", "));
    println!("  Integration options: {}", integration_options.join(", "));
    println!("  Minimal modes: {}", minimal_options.iter().map(|b| if *b { "minimal" } else { "full" }).collect::<Vec<_>>().join(", "));
    
    // Generate all combinations
    for fuzzer in &fuzzer_options {
        for integration in &integration_options {
            for &minimal in &minimal_options {
                let combination_name = format!(
                    "{}-{}-{}-{}",
                    template_name,
                    fuzzer,
                    integration,
                    if minimal { "minimal" } else { "full" }
                );
                let combination_path = test_dir.join(&combination_name);
                
                println!("    Generating: {}", combination_name);
                
                // Generate this specific combination
                let mut handlebars = Handlebars::new();
                
                // Register the 'eq' helper for conditional templating
                handlebars.register_helper("eq", Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
                    let param0 = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
                    let param1 = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
                    if param0 == param1 {
                        out.write("true")?;
                    }
                    Ok(())
                }));
                
                let project_name = format!("test-{}", template_name);
                let data = json!({ 
                    "project_name": project_name,
                    "target_name": project_name,
                    "default_fuzzer": fuzzer,
                    "integration": integration,
                    "minimal": minimal
                });
                
                // Clean up any existing combination directory
                if combination_path.exists() {
                    fs::remove_dir_all(&combination_path)?;
                }
                
                // Generate template
                process_template_directory(template_name, &combination_path, &handlebars, &data, metadata.as_ref())?;
                
                combinations.push((combination_name, combination_path));
            }
        }
    }
    
    Ok(combinations)
}

async fn test_combination_build(combination_path: &Path) -> anyhow::Result<bool> {
    // Determine build strategy based on what files exist
    let fuzz_dir = combination_path.join("fuzz");
    
    // Try different build methods in order of preference
    
    // 1. Try Makefile in root directory (full template)
    let root_makefile = combination_path.join("Makefile");
    if root_makefile.exists() {
        println!("    Using root Makefile build");
        let output = Command::new("make")
            .arg("fuzz")
            .current_dir(combination_path)
            .output();
        
        return match output {
            Ok(output) => {
                let success = output.status.success();
                if !success {
                    println!("    Build stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(success)
            }
            Err(e) => {
                println!("    Build error: {}", e);
                Ok(false)
            }
        };
    }
    
    // 2. Try CMake in root directory (full template)
    let root_cmake = combination_path.join("CMakeLists.txt");
    if root_cmake.exists() {
        println!("    Using root CMake build");
        let build_dir = combination_path.join("build");
        fs::create_dir_all(&build_dir)?;
        
        // Configure
        let configure_output = Command::new("cmake")
            .arg("..")
            .env("CC", "clang")  // Use clang for libFuzzer support
            .current_dir(&build_dir)
            .output();
        
        if let Ok(output) = configure_output {
            if output.status.success() {
                // Build
                let build_output = Command::new("cmake")
                    .arg("--build")
                    .arg(".")
                    .arg("--target")
                    .arg("fuzz")
                    .current_dir(&build_dir)
                    .output();
                
                return match build_output {
                    Ok(output) => {
                        let success = output.status.success();
                        if !success {
                            println!("    Build stderr: {}", String::from_utf8_lossy(&output.stderr));
                        }
                        Ok(success)
                    }
                    Err(e) => {
                        println!("    Build error: {}", e);
                        Ok(false)
                    }
                };
            }
        }
    }
    
    // 3. Try build script in fuzz directory
    let build_script = fuzz_dir.join("build.sh");
    if build_script.exists() {
        println!("    Using fuzz/build.sh");
        let output = Command::new("bash")
            .arg("build.sh")
            .current_dir(&fuzz_dir)
            .output();
        
        return match output {
            Ok(output) => {
                let success = output.status.success();
                if !success {
                    println!("    Build stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(success)
            }
            Err(e) => {
                println!("    Build error: {}", e);
                Ok(false)
            }
        };
    }
    
    // 4. Try Makefile in fuzz directory
    let fuzz_makefile = fuzz_dir.join("Makefile");
    if fuzz_makefile.exists() {
        println!("    Using fuzz/Makefile");
        
        // First try to build the library if we're in a full template
        let root_makefile = combination_path.join("Makefile");
        if root_makefile.exists() {
            let lib_output = Command::new("make")
                .arg("lib")
                .current_dir(combination_path)
                .output();
            
            if let Ok(output) = lib_output {
                if !output.status.success() {
                    println!("    Library build failed: {}", String::from_utf8_lossy(&output.stderr));
                    return Ok(false);
                }
            }
        }
        
        // Then build the fuzzer
        let output = Command::new("make")
            .current_dir(&fuzz_dir)
            .output();
        
        return match output {
            Ok(output) => {
                let success = output.status.success();
                if !success {
                    println!("    Build stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
                Ok(success)
            }
            Err(e) => {
                println!("    Build error: {}", e);
                Ok(false)
            }
        };
    }
    
    // 5. Try CMake in fuzz directory
    let fuzz_cmake = fuzz_dir.join("CMakeLists.txt");
    if fuzz_cmake.exists() {
        println!("    Using fuzz/CMakeLists.txt");
        let build_dir = fuzz_dir.join("build");
        fs::create_dir_all(&build_dir)?;
        
        // Configure
        let configure_output = Command::new("cmake")
            .arg("..")
            .env("CC", "clang")  // Use clang for libFuzzer support
            .current_dir(&build_dir)
            .output();
        
        if let Ok(output) = configure_output {
            if output.status.success() {
                // Build
                let build_output = Command::new("cmake")
                    .arg("--build")
                    .arg(".")
                    .current_dir(&build_dir)
                    .output();
                
                return match build_output {
                    Ok(output) => {
                        let success = output.status.success();
                        if !success {
                            println!("    Build stderr: {}", String::from_utf8_lossy(&output.stderr));
                        }
                        Ok(success)
                    }
                    Err(e) => {
                        println!("    Build error: {}", e);
                        Ok(false)
                    }
                };
            }
        }
    }
    
    // 6. For simple templates without build systems, just check if files were created
    if combination_path.exists() && fuzz_dir.exists() {
        println!("    Simple template - checking file generation only");
        return Ok(true);
    }
    
    println!("    No build system found");
    Ok(false)
}