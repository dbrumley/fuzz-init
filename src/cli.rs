use crate::types::*;
use crate::github_fetcher::fetch_github_template;
use anyhow;
use clap::Parser;
use inquire::{Select, Text};
use tempfile::TempDir;

#[derive(Parser)]
#[command(name = "fuzz-init")]
#[command(about = "Scaffold fuzz harnesses with Mayhem for various languages")]
#[command(version)]
pub struct Args {
    /// Positional argument for project name. 
    #[arg()]
    pub project_name_pos: Option<String>,

    /// Named flag alternative to positional.
    #[arg(long)]
    pub project: Option<String>,
    
    /// Programming language for the template (c, cpp, python, rust)
    #[arg(long)]
    pub language: Option<String>,
    
    /// Build system integration type (standalone, makefile, cmake)
    #[arg(long)]
    pub integration: Option<String>,
    
    /// Fuzzer type to use as default (afl, libfuzzer, honggfuzz, standalone)
    #[arg(long)]
    pub fuzzer: Option<String>,
    
    /// Template to use (github:org/repo, or @org/repo)
    #[arg(long)]
    pub template: Option<String>,
    
    /// Generate minimal template (fuzz directory only) instead of full tutorial
    #[arg(long)]
    pub minimal: bool,
}

pub fn get_project_name(args: &Args) -> anyhow::Result<String> {
    match args.project.as_ref().or(args.project_name_pos.as_ref()) {
        Some(name) => Ok(name.clone()),
        None => Ok(Text::new("Project name:").prompt()?),
    }
}

pub fn determine_template_source(args: &Args, available_templates: &[String]) -> anyhow::Result<TemplateSource> {
    match (&args.language, &args.template) {
        // Language specified - use local template
        (Some(language), None) => {
            if !available_templates.contains(language) {
                anyhow::bail!("Invalid language '{}'. Available: {}", language, available_templates.join(", "));
            }
            Ok(TemplateSource::Local(language.clone()))
        }
        // Template specified - use remote template
        (None, Some(template)) => {
            if template.starts_with("github:") {
                Ok(TemplateSource::GitHubFull(template.clone()))
            } else if template.starts_with('@') {
                Ok(TemplateSource::GitHubFull(template.clone()))
            } else {
                // Treat as local template name with deprecation warning
                println!("⚠️  Using --template for local templates is deprecated. Use --language instead.");
                if !available_templates.contains(template) {
                    anyhow::bail!("Invalid template '{}'. Available: {}", template, available_templates.join(", "));
                }
                Ok(TemplateSource::Local(template.clone()))
            }
        }
        // Both specified - error
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both --language and --template. Use --language for local templates or --template for remote templates.");
        }
        // Neither specified - prompt user
        (None, None) => {
            let selected = Select::new("Choose a language", available_templates.to_vec()).prompt()?;
            Ok(TemplateSource::Local(selected))
        }
    }
}

pub async fn get_template_name(template_source: &TemplateSource, available_templates: &[String]) -> anyhow::Result<(String, Option<TempDir>)> {
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
            let _path = temp_dir.path().to_path_buf();
            
            // For remote templates, we'll use the old filesystem-based approach
            // until we implement embedding for remote templates too
            anyhow::bail!("Remote templates not yet supported with embedded template system. Use local templates only.")
        }
    }
}

pub fn select_fuzzer(args: &Args, metadata: Option<&TemplateMetadata>) -> anyhow::Result<String> {
    if let Some(fuzzer) = &args.fuzzer {
        // Validate fuzzer type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(fuzzers) = &metadata.fuzzers {
                if !fuzzers.supported.contains(fuzzer) {
                    anyhow::bail!("Fuzzer '{}' not supported by this template. Supported: {}", 
                                fuzzer, fuzzers.supported.join(", "));
                }
            }
        }
        Ok(fuzzer.clone())
    } else {
        // Get default from metadata or prompt user
        if let Some(metadata) = metadata {
            if let Some(fuzzers) = &metadata.fuzzers {
                if fuzzers.supported.len() == 1 {
                    // Only one option, use it
                    Ok(fuzzers.supported[0].clone())
                } else {
                    // Multiple options, prompt user
                    let options: Vec<String> = fuzzers.options.iter()
                        .map(|opt| format!("{} - {}", opt.display_name, opt.description))
                        .collect();
                    let selected = Select::new("Choose a fuzzer", options).prompt()?;
                    let fuzzer_name = selected.split(" - ").next().unwrap();
                    
                    // Find the actual fuzzer name from display name
                    for option in &fuzzers.options {
                        if option.display_name == fuzzer_name {
                            return Ok(option.name.clone());
                        }
                    }
                    Ok(fuzzers.default.clone())
                }
            } else {
                Ok("libfuzzer".to_string()) // Default fallback
            }
        } else {
            Ok("libfuzzer".to_string()) // Default fallback
        }
    }
}

pub fn select_integration(args: &Args, metadata: Option<&TemplateMetadata>) -> anyhow::Result<String> {
    if let Some(integration) = &args.integration {
        // Validate integration type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(integrations) = &metadata.integrations {
                if !integrations.supported.contains(integration) {
                    anyhow::bail!("Integration '{}' not supported by this template. Supported: {}", 
                                integration, integrations.supported.join(", "));
                }
            }
        }
        Ok(integration.clone())
    } else {
        // Get default from metadata or prompt user
        if let Some(metadata) = metadata {
            if let Some(integrations) = &metadata.integrations {
                if integrations.supported.len() == 1 {
                    // Only one option, use it
                    Ok(integrations.supported[0].clone())
                } else {
                    // Multiple options, prompt user
                    let options: Vec<String> = integrations.options.iter()
                        .map(|opt| format!("{} - {}", opt.name, opt.description))
                        .collect();
                    let selected = Select::new("Choose build system integration", options).prompt()?;
                    let integration_name = selected.split(" - ").next().unwrap();
                    Ok(integration_name.to_string())
                }
            } else {
                Ok("make".to_string()) // Default fallback
            }
        } else {
            Ok("make".to_string()) // Default fallback
        }
    }
}

pub fn determine_minimal_mode(args: &Args, _template_source: &TemplateSource) -> bool {
    args.minimal
}

pub fn print_next_steps(project_name: &str, default_fuzzer: &str, integration_type: &str, minimal_mode: bool, metadata: Option<&TemplateMetadata>) {
    println!();
    println!("🚀 Next steps:");
    println!("==============");
    println!("1. cd {}", project_name);
    
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
            println!("2. cd fuzz && ./build.sh  # Build the fuzzer");
        }
        _ => {
            println!("2. See project documentation for build instructions");
        }
    }
    
    println!();
    println!("🔍 Run your fuzzer:");
    match integration_type {
        "cmake" => {
            match default_fuzzer {
                "libfuzzer" => println!("   cd fuzz/build && ./{}_libfuzzer ../testsuite/", project_name),
                "afl" => println!("   cd fuzz/build && ./{}_afl ../testsuite/", project_name),
                "honggfuzz" => println!("   cd fuzz/build && ./{}_honggfuzz ../testsuite/", project_name),
                _ => println!("   cd fuzz/build && ./{}_{}  ../testsuite/", project_name, default_fuzzer),
            }
        }
        "make" => {
            println!("   cd fuzz && ./{}-{}  testsuite/", project_name, default_fuzzer);
        }
        "standalone" => {
            println!("   cd fuzz && ./bin/{}  testsuite/", project_name);
        }
        _ => {
            println!("   See project documentation for usage instructions");
        }
    }
    
    println!();
    println!("📚 Documentation:");
    if !minimal_mode {
        println!("   • TUTORIAL.md          - Complete fuzzing tutorial and examples");
    }
    println!("   • fuzz/INTEGRATION.md  - Integration guide for existing projects");
    println!("   • fuzz/README.md       - Quick reference for fuzzing commands");
    
    if let Some(metadata) = metadata {
        println!();
        println!("💡 Template info:");
        println!("   • Language: {}", metadata.template.name);
        println!("   • Description: {}", metadata.template.description);
        
        if let Some(fuzzers) = &metadata.fuzzers {
            let other_fuzzers: Vec<String> = fuzzers.supported.iter()
                .filter(|&f| f != default_fuzzer)
                .cloned()
                .collect();
            if !other_fuzzers.is_empty() {
                println!("   • Other fuzzer types available: {}", other_fuzzers.join(", "));
            }
        }
    }
    
    println!();
    println!("Happy fuzzing! 🐛");
}