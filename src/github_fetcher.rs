use crate::types::*;
use anyhow;
use reqwest;
use std::{fs, path::Path};
use tempfile::TempDir;

pub async fn fetch_github_template(source: &TemplateSource) -> anyhow::Result<TempDir> {
    let (org, repo, path) = match source {
        TemplateSource::GitHub { org, repo, path } => (org.clone(), repo.clone(), path.clone()),
        TemplateSource::GitHubFull(spec) => {
            if spec.starts_with("github:") {
                let spec = &spec[7..]; // Remove "github:" prefix
                let parts: Vec<&str> = spec.split('/').collect();
                if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1].to_string(), None)
                } else {
                    anyhow::bail!("Invalid GitHub template format: {}", spec);
                }
            } else if spec.starts_with('@') {
                let spec = &spec[1..]; // Remove "@" prefix
                let parts: Vec<&str> = spec.split('/').collect();
                if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1].to_string(), None)
                } else {
                    anyhow::bail!("Invalid GitHub template format: {}", spec);
                }
            } else {
                anyhow::bail!("Invalid GitHub template format: {}", spec);
            }
        }
        _ => anyhow::bail!("Not a GitHub template source"),
    };

    let temp_dir = TempDir::new()?;
    
    // Download the repository as a ZIP file
    let url = format!("https://github.com/{}/{}/archive/main.zip", org, repo);
    println!("Fetching template from {}", url);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        // Try with 'master' branch instead of 'main'
        let url = format!("https://github.com/{}/{}/archive/master.zip", org, repo);
        println!("Retrying with master branch: {}", url);
        let response = client.get(&url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch template: HTTP {}", response.status());
        }
    }
    
    let zip_content = response.bytes().await?;
    
    // Extract the ZIP file
    let cursor = std::io::Cursor::new(zip_content);
    let mut archive = zip::ZipArchive::new(cursor)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.mangled_name();
        
        // Skip directories
        if file_path.to_string_lossy().ends_with('/') {
            continue;
        }
        
        // Extract to temp directory, removing the top-level directory from the archive
        let mut path_parts: Vec<_> = file_path.components().collect();
        if !path_parts.is_empty() {
            path_parts.remove(0); // Remove the top-level directory (repo-main or repo-master)
        }
        
        if path_parts.is_empty() {
            continue;
        }
        
        let output_path = temp_dir.path().join(path_parts.iter().collect::<std::path::PathBuf>());
        
        // Create parent directories
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Extract the file
        let mut output_file = fs::File::create(&output_path)?;
        std::io::copy(&mut file, &mut output_file)?;
    }
    
    // If a specific path was requested, move that subdirectory to the root
    if let Some(subpath) = path {
        let source_path = temp_dir.path().join(&subpath);
        if source_path.exists() {
            let new_temp_dir = TempDir::new()?;
            copy_dir_recursive(&source_path, new_temp_dir.path())?;
            return Ok(new_temp_dir);
        } else {
            anyhow::bail!("Specified path '{}' not found in repository", subpath);
        }
    }
    
    Ok(temp_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}