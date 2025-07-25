use std::{fs, path::{Path, PathBuf}, collections::HashMap, process::Command};
use inquire::{Text, Select};
use handlebars::Handlebars;
use serde_json::json;
use clap::Parser;
use serde::{Deserialize, Serialize};
use reqwest;
use tempfile::TempDir;

fn get_available_templates() -> anyhow::Result<Vec<String>> {
    let templates_dir = "src/templates";
    let mut templates = Vec::new();
    
    for entry in fs::read_dir(templates_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                templates.push(name.to_string());
            }
        }
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
        .header("User-Agent", "mayhem-init")
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

fn load_template_metadata(template_dir: &Path) -> anyhow::Result<Option<TemplateMetadata>> {
    let metadata_path = template_dir.join("template.toml");
    if metadata_path.exists() {
        let content = fs::read_to_string(&metadata_path)?;
        let metadata: TemplateMetadata = toml::from_str(&content)?;
        Ok(Some(metadata))
    } else {
        Ok(None)
    }
}

fn get_file_config<'a>(metadata: Option<&'a TemplateMetadata>, relative_path: &str) -> Option<&'a FileConfig> {
    metadata?.files.iter().find(|f| f.path == relative_path)
}

fn should_include_file(metadata: Option<&TemplateMetadata>, relative_path: &str, data: &serde_json::Value) -> bool {
    if let Some(config) = get_file_config(metadata, relative_path) {
        if let Some(condition) = &config.condition {
            return evaluate_condition(condition, data);
        }
    }
    true // Include by default if no condition
}

fn evaluate_condition(condition: &str, data: &serde_json::Value) -> bool {
    // Handle OR conditions
    if condition.contains("||") {
        return condition.split("||")
            .map(|part| part.trim())
            .any(|part| evaluate_single_condition(part, data));
    }
    
    // Handle AND conditions
    if condition.contains("&&") {
        return condition.split("&&")
            .map(|part| part.trim())
            .all(|part| evaluate_single_condition(part, data));
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
    if let Some(config) = get_file_config(metadata, relative_path) {
        config.template
    } else {
        // Default behavior: template text files
        is_text_file(file_path)
    }
}

fn should_be_executable(metadata: Option<&TemplateMetadata>, relative_path: &str, file_path: &Path) -> anyhow::Result<bool> {
    if let Some(config) = get_file_config(metadata, relative_path) {
        Ok(config.executable)
    } else {
        // Default behavior: check existing permissions
        is_executable(file_path)
    }
}

fn process_template_directory(
    template_dir: &Path,
    output_dir: &Path,
    handlebars: &Handlebars,
    data: &serde_json::Value,
    metadata: Option<&TemplateMetadata>,
) -> anyhow::Result<()> {
    process_template_directory_impl(template_dir, output_dir, handlebars, data, metadata, template_dir)
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

#[derive(Parser)]
#[command(name = "mayhem-init")]
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

async fn get_template_directory(template_source: &TemplateSource, available_templates: &[String]) -> anyhow::Result<(PathBuf, Option<TempDir>)> {
    match template_source {
        TemplateSource::Local(name) => {
            if !available_templates.contains(name) {
                anyhow::bail!("Invalid template name. Available templates: {}", available_templates.join(", "));
            }
            let path = std::env::current_dir()?.join("src/templates").join(name);
            Ok((path, None))
        }
        _ => {
            let temp_dir = fetch_github_template(template_source).await?;
            let path = temp_dir.path().to_path_buf();
            Ok((path, Some(temp_dir)))
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

fn get_template_name(template_source: &TemplateSource) -> String {
    match template_source {
        TemplateSource::Local(name) => name.clone(),
        TemplateSource::GitHub { org, repo, .. } => format!("{}/{}", org, repo),
        TemplateSource::GitHubFull(spec) => spec.clone(),
    }
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
    let (template_dir, _temp_dir) = get_template_directory(&template_source, &available_templates).await?;
    
    // Load template metadata
    let metadata = load_template_metadata(&template_dir)?;
    
    // Get user selections
    let default_fuzzer = select_fuzzer(&args, metadata.as_ref())?;
    let integration_type = select_integration(&args, metadata.as_ref())?;
    let minimal_mode = determine_minimal_mode(&args, &template_source);
    
    // Create template data
    let handlebars = Handlebars::new();
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
    process_template_directory(&template_dir, out_path, &handlebars, &data, metadata.as_ref())?;
    
    // Success message
    let template_name = get_template_name(&template_source);
    println!("Project '{}' created with {} template!", project_name, template_name);
    Ok(())
}

async fn run_template_tests() -> anyhow::Result<()> {
    println!("🧪 Running template tests...\n");
    
    let available_templates = get_available_templates()?;
    let mut test_results = Vec::new();
    
    for template_name in &available_templates {
        println!("Testing template: {}", template_name);
        let result = test_template(template_name).await;
        test_results.push((template_name.clone(), result));
        println!();
    }
    
    // Print summary
    println!("📊 Test Summary:");
    println!("================");
    let mut passed = 0;
    let mut failed = 0;
    
    for (template_name, result) in &test_results {
        match result {
            Ok(fuzzer_results) => {
                let fuzzer_passed = fuzzer_results.iter().filter(|(_, success)| *success).count();
                let fuzzer_total = fuzzer_results.len();
                
                if fuzzer_passed == fuzzer_total {
                    println!("✅ {} - All {} fuzzer modes passed", template_name, fuzzer_total);
                    passed += 1;
                } else {
                    println!("❌ {} - {}/{} fuzzer modes passed", template_name, fuzzer_passed, fuzzer_total);
                    for (fuzzer_type, success) in fuzzer_results {
                        if !success {
                            println!("   └─ ❌ {} failed", fuzzer_type);
                        }
                    }
                    failed += 1;
                }
            }
            Err(e) => {
                println!("❌ {} - Template failed: {}", template_name, e);
                failed += 1;
            }
        }
    }
    
    println!("\nFinal Result: {}/{} templates passed", passed, passed + failed);
    
    if failed > 0 {
        anyhow::bail!("Some templates failed testing");
    }
    
    Ok(())
}

async fn test_template(template_name: &str) -> anyhow::Result<Vec<(String, bool)>> {
    // Create temporary directory for testing
    let temp_dir = tempfile::tempdir()?;
    let test_project_name = format!("test-{}", template_name);
    let test_project_path = temp_dir.path().join(&test_project_name);
    
    // Load template metadata to get fuzzer options
    let template_dir = std::env::current_dir()?.join("src/templates").join(template_name);
    let metadata = load_template_metadata(&template_dir)?;
    
    let fuzzer_options = if let Some(ref metadata) = metadata {
        if let Some(ref fuzzer_config) = metadata.fuzzers {
            fuzzer_config.options.iter().map(|opt| opt.name.clone()).collect()
        } else {
            vec!["standalone".to_string()]
        }
    } else {
        vec!["standalone".to_string()]
    };
    
    println!("  Fuzzer options: {}", fuzzer_options.join(", "));
    
    let mut fuzzer_results = Vec::new();
    
    for fuzzer_type in &fuzzer_options {
        println!("  Testing fuzzer: {}", fuzzer_type);
        
        // Generate template with this fuzzer as default
        let handlebars = Handlebars::new();
        let data = json!({ 
            "project_name": test_project_name,
            "target_name": test_project_name,
            "default_fuzzer": fuzzer_type
        });
        
        // Clean up any existing test project
        if test_project_path.exists() {
            fs::remove_dir_all(&test_project_path)?;
        }
        
        // Generate template
        process_template_directory(&template_dir, &test_project_path, &handlebars, &data, metadata.as_ref())?;
        
        // Test if this template can build
        let success = test_template_build(&test_project_path, fuzzer_type).await?;
        fuzzer_results.push((fuzzer_type.clone(), success));
        
        if success {
            println!("    ✅ Build successful");
        } else {
            println!("    ❌ Build failed");
        }
    }
    
    Ok(fuzzer_results)
}

async fn test_template_build(project_path: &Path, fuzzer_type: &str) -> anyhow::Result<bool> {
    // Look for build script
    let build_script = project_path.join("fuzz").join("build.sh");
    if !build_script.exists() {
        // For simple templates without build scripts, just check if files were created
        return Ok(project_path.exists());
    }
    
    // Change to project directory and run build
    let output = Command::new("bash")
        .arg("build.sh")
        .current_dir(project_path.join("fuzz"))
        .env("FUZZER_TYPE", fuzzer_type)
        .output();
    
    match output {
        Ok(output) => {
            let success = output.status.success();
            if !success {
                println!("    Build stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(success)
        }
        Err(e) => {
            println!("    Build error: {}", e);
            Ok(false) // Don't fail the entire test, just mark this fuzzer as failed
        }
    }
}