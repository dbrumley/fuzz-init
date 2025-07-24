use std::{fs, path::Path, collections::HashMap};
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
struct TemplateMetadata {
    template: TemplateInfo,
    variables: HashMap<String, VariableConfig>,
    #[serde(default)]
    files: Vec<FileConfig>,
    #[serde(default)]
    directories: Vec<DirectoryConfig>,
    #[serde(default)]
    hooks: HookConfig,
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
    
    /// Template to use (local name, github:org/repo, or @org/repo)
    #[arg(long)]
    template: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args = Args::parse();
    
    // Get available templates
    let available_templates = get_available_templates()?;
    if available_templates.is_empty() {
        anyhow::bail!("No templates found in src/templates directory");
    }
    
    // Get project name - use provided or prompt
    let project_name = match args.project.or(args.project_name_pos) {
        Some(name) => name,
        None => Text::new("Project name:").prompt()?,
    };

    
    // Handle template source - local or remote
    let template_source = match args.template {
        Some(template_str) => TemplateSource::parse(&template_str)?,
        None => {
            // Prompt user to choose from available local templates
            let selected = Select::new("Choose a template", available_templates.clone()).prompt()?;
            TemplateSource::Local(selected)
        }
    };

    // Get template directory (either local or downloaded from remote)
    let (template_dir, _temp_dir) = match &template_source {
        TemplateSource::Local(name) => {
            if !available_templates.contains(name) {
                anyhow::bail!("Invalid template name. Available templates: {}", available_templates.join(", "));
            }
            let path = std::env::current_dir()?.join("src/templates").join(name);
            (path, None)
        }
        _ => {
            let temp_dir = fetch_github_template(&template_source).await?;
            let path = temp_dir.path().to_path_buf();
            (path, Some(temp_dir))
        }
    };

    let out_path_string = format!("./{}", project_name);
    let out_path = Path::new(&out_path_string);

    // Load template metadata if available
    let metadata = load_template_metadata(&template_dir)?;

    let handlebars = Handlebars::new();
    let data = json!({ 
        "project_name": project_name,
        "target_name": project_name // Use project name as target name by default
    });

    // Use recursive templating engine with metadata
    process_template_directory(&template_dir, &out_path, &handlebars, &data, metadata.as_ref())?;

    // Determine template name for output message
    let template_name = match &template_source {
        TemplateSource::Local(name) => name.clone(),
        TemplateSource::GitHub { org, repo, .. } => format!("{}/{}", org, repo),
        TemplateSource::GitHubFull(spec) => spec.clone(),
    };

    println!("Project '{}' created with {} template!", project_name, template_name);
    Ok(())
}