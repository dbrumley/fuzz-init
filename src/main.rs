use std::path::Path;
use serde_json::json;
use clap::Parser;

mod types;
mod template_processor;
mod github_fetcher;
mod cli;

// use types::*; // Not needed in main
use template_processor::*;
use cli::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Get available templates
    let available_templates = get_available_templates()?;
    if available_templates.is_empty() {
        anyhow::bail!("No templates found in embedded templates");
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
    
    // Setup Handlebars with helpers
    let handlebars = setup_handlebars();
    
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