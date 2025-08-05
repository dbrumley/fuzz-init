use crate::github_fetcher::fetch_github_template;
use crate::types::*;
use anyhow;
use clap::Parser;
use inquire::{Select, Text};
use std::path::PathBuf;

/// Find a template name case-insensitively and return the actual template name
fn find_template_case_insensitive(
    input_name: &str,
    available_templates: &[String],
) -> Option<String> {
    available_templates
        .iter()
        .find(|template| template.to_lowercase() == input_name.to_lowercase())
        .cloned()
}


#[derive(Parser)]
#[command(name = "fuzz-init")]
#[command(
    about = "🚀 Scaffold fuzz harnesses using best practices fast",
    long_about = "🚀 Scaffold fuzz harnesses using best practices fast.\n\n\
  ✨ Features:\n\
    - 🛠️ Languages: c, cpp, rust, python, ...\n\
    - 🧱 Build systems: make, cmake, cargo, ...\n\
    - 🐛 Fuzzers: libfuzzer, afl, honggfuzz\n\
    - 📦 Templates: extend with custom templates from GitHub or local files\n\n\
  📚 Choose between:\n\
    → Full tutorial project with examples and docs (default)\n\
    → Minimal fuzz/ folder for integrating into existing codebases\n\n\
"
)]
#[command(version)]
pub struct Args {
    #[arg(
        value_name = "PROJECT_NAME",
        long_help = "📛 Project name for the generated fuzzing project.\n\
Used as both the output directory and the default target name.\n\n\
Examples:\n  - fuzz-init my-project\n  - fuzz-init http-parser"
    )]
    pub project_name_pos: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_PROJECT",
        value_name = "PROJECT_NAME",
        long_help = "🏷️ Explicit project name (same as positional PROJECT_NAME).\n\
Useful when scripting or avoiding positional args.\n\n\
Example:\n  - fuzz-init --project my-project"
    )]
    pub project: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_LANG",
        value_name = "LANG",
        long_help = "🧠 Language for the fuzzing template.\n\
Each template includes language-specific integration and examples.\n\n\
Examples:\n  - fuzz-init --language c\n  - fuzz-init --language rust"
    )]
    pub language: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_INTEGRATION",
        value_name = "TYPE",
        long_help = "🔧 Build system integration\n\
Defines how the fuzzing infrastructure plugs into your build.\n\n\
Examples:\n  - --integration cmake\n  - --integration make"
    )]
    pub integration: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_FUZZER",
        value_name = "FUZZER",
        long_help = "🐞 Fuzzer engine to configure\n\n\
All templates use LLVMFuzzerTestOneInput-style harnesses. This flag\ncustomizes the build setup.\n\n\
Examples:\n  - --fuzzer libfuzzer\n  - --fuzzer afl"
    )]
    pub fuzzer: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_TEMPLATE",
        value_name = "SOURCE",
        long_help = "🌐 Remote template instead of built-in language templates\n\n\
Examples:\n  - --template github:forallsecure/c-fuzzing-template\n  - --template @myorg/custom-template"
    )]
    #[arg(long, value_name = "SOURCE")]
    pub template: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_MINIMAL",
        long_help = "✂️ Generate minimal project (fuzz/ only)\n\n\
Choose this for drop-in harnesses instead of full examples/tutorials.\n\n\
Usage:\n  - fuzz-init --minimal"
    )]
    #[arg(long)]
    pub minimal: bool,

    #[arg(
        long,
        hide = false,
        env = "FUZZ_INIT_GENERATE_DOCS",
        long_help = "Generate comprehensive CLI documentation in MDX format."
    )]
    pub generate_docs: bool,

    #[arg(
        long,
        env = "FUZZ_INIT_DEV_MODE",
        long_help = "🔧 Template development mode with multi-config testing and validation\n\n\
Runs comprehensive testing of template configurations:\n\
  - Tests all fuzzer×integration×mode combinations\n\
  - Shows detailed build results and error reporting\n\
  - Uses temporary directories for isolated testing\n\n\
Examples:\n\
  - fuzz-init --dev-mode --language C\n\
  - fuzz-init --dev-mode --language C --watch src/templates/C/"
    )]
    pub dev_mode: bool,

    #[arg(
        long,
        env = "FUZZ_INIT_WATCH",
        value_name = "PATH",
        long_help = "👀 Watch template directory for changes and auto-rebuild\n\n\
Monitors template files and re-runs validation when changes detected.\n\
Must be used with --dev-mode.\n\n\
Examples:\n\
  - --watch src/templates/C/\n\
  - --watch /path/to/custom/template/"
    )]
    pub watch: Option<String>,

    #[arg(
        long,
        env = "FUZZ_INIT_DEV_OUTPUT",
        value_name = "DIR",
        long_help = "📁 Custom output directory for development testing\n\n\
By default, uses temporary directories that are auto-cleaned.\n\
Specify this to use a persistent location for debugging.\n\n\
Example:\n\
  - --dev-output ./template-testing/"
    )]
    pub dev_output: Option<String>,
}

pub fn get_project_name(args: &Args) -> anyhow::Result<String> {
    match args.project.as_ref().or(args.project_name_pos.as_ref()) {
        Some(name) => Ok(name.clone()),
        None => Ok(Text::new("Project name:").prompt()?),
    }
}

pub fn get_project_name_with_tracking(args: &Args) -> anyhow::Result<(String, bool)> {
    match args.project.as_ref().or(args.project_name_pos.as_ref()) {
        Some(name) => Ok((name.clone(), false)), // false = not prompted
        None => {
            let name = Text::new("Project name:").prompt()?;
            Ok((name, true)) // true = prompted
        }
    }
}

pub fn determine_template_source(
    args: &Args,
    available_templates: &[String],
) -> anyhow::Result<TemplateSource> {
    match (&args.language, &args.template) {
        // Language specified - use local template
        (Some(language), None) => {
            if let Some(actual_template_name) = find_template_case_insensitive(language, available_templates) {
                Ok(TemplateSource::Local(actual_template_name))
            } else {
                anyhow::bail!(
                    "Invalid language '{}'. Available: {}",
                    language,
                    available_templates.join(", ")
                );
            }
        }
        // Template specified - use remote template
        (None, Some(template)) => {
            if template.starts_with("github:") {
                Ok(TemplateSource::GitHubFull(template.clone()))
            } else if template.starts_with('@') {
                Ok(TemplateSource::GitHubFull(template.clone()))
            } else {
                // Treat as local template name with deprecation warning
                // println!("⚠️  Using --template for local templates is deprecated. Use --language instead.");
                if let Some(actual_template_name) = find_template_case_insensitive(template, available_templates) {
                    Ok(TemplateSource::Local(actual_template_name))
                } else {
                    anyhow::bail!(
                        "Invalid template '{}'. Available: {}",
                        template,
                        available_templates.join(", ")
                    );
                }
            }
        }
        // Both specified - error
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both --language and --template. Use --language for local templates or --template for remote templates.");
        }
        // Neither specified - prompt user
        (None, None) => {
            let selected =
                Select::new("Choose a language", available_templates.to_vec()).prompt()?;
            Ok(TemplateSource::Local(selected))
        }
    }
}

pub fn determine_template_source_with_tracking(
    args: &Args,
    available_templates: &[String],
) -> anyhow::Result<(TemplateSource, bool)> {
    match (&args.language, &args.template) {
        // Language specified - use local template
        (Some(language), None) => {
            if let Some(actual_template_name) = find_template_case_insensitive(language, available_templates) {
                Ok((TemplateSource::Local(actual_template_name), false))
            } else {
                anyhow::bail!(
                    "Invalid language '{}'. Available: {}",
                    language,
                    available_templates.join(", ")
                );
            }
        }
        // Template specified - use remote template
        (None, Some(template)) => {
            if template.starts_with("github:") {
                Ok((TemplateSource::GitHubFull(template.clone()), false))
            } else if template.starts_with('@') {
                Ok((TemplateSource::GitHubFull(template.clone()), false))
            } else {
                if let Some(actual_template_name) = find_template_case_insensitive(template, available_templates) {
                    Ok((TemplateSource::Local(actual_template_name), false))
                } else {
                    anyhow::bail!(
                        "Invalid template '{}'. Available: {}",
                        template,
                        available_templates.join(", ")
                    );
                }
            }
        }
        // Both specified - error
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both --language and --template. Use --language for local templates or --template for remote templates.");
        }
        // Neither specified - prompt user
        (None, None) => {
            let selected =
                Select::new("Choose a language", available_templates.to_vec()).prompt()?;
            Ok((TemplateSource::Local(selected), true)) // true = prompted
        }
    }
}

pub async fn get_template_name(
    template_source: &TemplateSource,
    available_templates: &[String],
) -> anyhow::Result<(String, Option<PathBuf>)> {
    match template_source {
        TemplateSource::Local(name) => {
            if !available_templates.contains(name) {
                anyhow::bail!(
                    "Invalid template name. Available templates: {}",
                    available_templates.join(", ")
                );
            };
            Ok((name.clone(), None))
        }
        _ => {
            // For remote templates, fetch them to temp directory
            let temp_dir = fetch_github_template(template_source).await?;
            
            // We need to keep the TempDir alive, so we leak it intentionally
            // The OS will clean it up when the process exits
            let path = temp_dir.keep();
            
            // Return a generic name and the path to the downloaded template
            Ok(("remote".to_string(), Some(path)))
        }
    }
}

pub fn select_fuzzer(args: &Args, metadata: Option<&TemplateMetadata>) -> anyhow::Result<String> {
    if let Some(fuzzer) = &args.fuzzer {
        // Validate fuzzer type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(fuzzers) = &metadata.fuzzers {
                if !fuzzers.supported.contains(fuzzer) {
                    anyhow::bail!(
                        "Fuzzer '{}' not supported by this template. Supported: {}",
                        fuzzer,
                        fuzzers.supported.join(", ")
                    );
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
                    let options: Vec<String> = fuzzers
                        .options
                        .iter()
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

pub fn select_fuzzer_with_tracking(args: &Args, metadata: Option<&TemplateMetadata>) -> anyhow::Result<(String, bool)> {
    if let Some(fuzzer) = &args.fuzzer {
        // Validate fuzzer type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(fuzzers) = &metadata.fuzzers {
                if !fuzzers.supported.contains(fuzzer) {
                    anyhow::bail!(
                        "Fuzzer '{}' not supported by this template. Supported: {}",
                        fuzzer,
                        fuzzers.supported.join(", ")
                    );
                }
            }
        }
        Ok((fuzzer.clone(), false)) // false = not prompted
    } else {
        // Get default from metadata or prompt user
        if let Some(metadata) = metadata {
            if let Some(fuzzers) = &metadata.fuzzers {
                if fuzzers.supported.len() == 1 {
                    // Only one option, use it (not considered prompted)
                    Ok((fuzzers.supported[0].clone(), false))
                } else {
                    // Multiple options, prompt user
                    let options: Vec<String> = fuzzers
                        .options
                        .iter()
                        .map(|opt| format!("{} - {}", opt.display_name, opt.description))
                        .collect();
                    let selected = Select::new("Choose a fuzzer", options).prompt()?;
                    let fuzzer_name = selected.split(" - ").next().unwrap();

                    // Find the actual fuzzer name from display name
                    for option in &fuzzers.options {
                        if option.display_name == fuzzer_name {
                            return Ok((option.name.clone(), true)); // true = prompted
                        }
                    }
                    Ok((fuzzers.default.clone(), true)) // true = prompted
                }
            } else {
                Ok(("libfuzzer".to_string(), false)) // Default fallback
            }
        } else {
            Ok(("libfuzzer".to_string(), false)) // Default fallback
        }
    }
}

pub fn select_integration(
    args: &Args,
    metadata: Option<&TemplateMetadata>,
) -> anyhow::Result<String> {
    if let Some(integration) = &args.integration {
        // Validate integration type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(integrations) = &metadata.integrations {
                if !integrations.supported.contains(integration) {
                    anyhow::bail!(
                        "Integration '{}' not supported by this template. Supported: {}",
                        integration,
                        integrations.supported.join(", ")
                    );
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
                    // Ensure default option is listed first
                    let mut sorted_options = integrations.options.clone();
                    
                    // Find the default option and move it to the front if it exists
                    if let Some(default_pos) = sorted_options.iter().position(|opt| opt.name == integrations.default) {
                        let default_option = sorted_options.remove(default_pos);
                        sorted_options.insert(0, default_option);
                    }
                    
                    let options: Vec<String> = sorted_options
                        .iter()
                        .map(|opt| format!("{} - {}", opt.name, opt.description))
                        .collect();

                    // Default is now always at index 0
                    let selected = Select::new("Choose build system integration", options)
                        .with_starting_cursor(0)
                        .prompt()?;
                    let integration_name = selected.split(" - ").next().unwrap();
                    Ok(integration_name.to_string())
                }
            } else {
                anyhow::bail!("Template does not define any integration options")
            }
        } else {
            anyhow::bail!("Template metadata is missing integration configuration")
        }
    }
}

pub fn select_integration_with_tracking(
    args: &Args,
    metadata: Option<&TemplateMetadata>,
) -> anyhow::Result<(String, bool)> {
    if let Some(integration) = &args.integration {
        // Validate integration type against template metadata if available
        if let Some(metadata) = metadata {
            if let Some(integrations) = &metadata.integrations {
                if !integrations.supported.contains(integration) {
                    anyhow::bail!(
                        "Integration '{}' not supported by this template. Supported: {}",
                        integration,
                        integrations.supported.join(", ")
                    );
                }
            }
        }
        Ok((integration.clone(), false)) // false = not prompted
    } else {
        // Get default from metadata or prompt user
        if let Some(metadata) = metadata {
            if let Some(integrations) = &metadata.integrations {
                if integrations.supported.len() == 1 {
                    // Only one option, use it (not considered prompted)
                    Ok((integrations.supported[0].clone(), false))
                } else {
                    // Multiple options, prompt user
                    // Ensure default option is listed first
                    let mut sorted_options = integrations.options.clone();
                    
                    // Find the default option and move it to the front if it exists
                    if let Some(default_pos) = sorted_options.iter().position(|opt| opt.name == integrations.default) {
                        let default_option = sorted_options.remove(default_pos);
                        sorted_options.insert(0, default_option);
                    }
                    
                    let options: Vec<String> = sorted_options
                        .iter()
                        .map(|opt| format!("{} - {}", opt.name, opt.description))
                        .collect();

                    // Default is now always at index 0
                    let selected = Select::new("Choose build system integration", options)
                        .with_starting_cursor(0)
                        .prompt()?;
                    let integration_name = selected.split(" - ").next().unwrap();
                    Ok((integration_name.to_string(), true)) // true = prompted
                }
            } else {
                anyhow::bail!("Template does not define any integration options")
            }
        } else {
            anyhow::bail!("Template metadata is missing integration configuration")
        }
    }
}

pub fn determine_minimal_mode(args: &Args, _template_source: &TemplateSource) -> bool {
    args.minimal
}

pub fn print_next_steps(
    project_name: &str, 
    minimal_mode: bool, 
    prompted_values: &crate::types::PromptedValues, 
    template_source: &TemplateSource, 
    fuzzer: &str,
    integration: &str
) {
    println!();
    println!("🚀 Next steps:");
    println!("==============");
    if !minimal_mode {
        println!("1. cd {}", project_name);
        println!("2. Read TUTORIAL.md");
    } else {
        println!("1. cd {}/fuzz", project_name);
        println!("2. Read INTEGRATION.md");
    }

    println!();
    println!("📚 Documentation:");
    if !minimal_mode {
        println!("   • TUTORIAL.md          - Complete fuzzing tutorial and examples");
    }
    println!("   - fuzz/INTEGRATION.md  - Integration guide for existing projects");
    println!("   - fuzz/README.md       - Quick reference for fuzzing commands");

    // Generate CLI hint if any values were prompted
    if prompted_values.project_name || prompted_values.language || prompted_values.fuzzer || prompted_values.integration {
        println!();
        println!("💡 CLI Hint:");
        println!("============");
        println!("To recreate this project without prompts, use:");
        println!();
        
        let mut command = "fuzz-init".to_string();
        
        // Add project name
        command.push_str(&format!(" {}", project_name));
        
        // Add language if it was from template source
        if let TemplateSource::Local(language) = template_source {
            command.push_str(&format!(" --language {}", language));
        }
        
        // Add other parameters
        command.push_str(&format!(" --fuzzer {}", fuzzer));
        command.push_str(&format!(" --integration {}", integration));
        
        if minimal_mode {
            command.push_str(" --minimal");
        }
        
        println!("  {}", command);
        println!();
    }

    println!("🐛 Happy fuzzing!");
}
