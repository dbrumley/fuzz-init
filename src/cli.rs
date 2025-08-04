use crate::github_fetcher::fetch_github_template;
use crate::types::*;
use anyhow;
use clap::Parser;
use inquire::{Select, Text};
use tempfile::TempDir;

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
}

pub fn get_project_name(args: &Args) -> anyhow::Result<String> {
    match args.project.as_ref().or(args.project_name_pos.as_ref()) {
        Some(name) => Ok(name.clone()),
        None => Ok(Text::new("Project name:").prompt()?),
    }
}

pub fn determine_template_source(
    args: &Args,
    available_templates: &[String],
) -> anyhow::Result<TemplateSource> {
    match (&args.language, &args.template) {
        // Language specified - use local template
        (Some(language), None) => {
            if !available_templates.contains(language) {
                anyhow::bail!(
                    "Invalid language '{}'. Available: {}",
                    language,
                    available_templates.join(", ")
                );
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
                // println!("⚠️  Using --template for local templates is deprecated. Use --language instead.");
                if !available_templates.contains(template) {
                    anyhow::bail!(
                        "Invalid template '{}'. Available: {}",
                        template,
                        available_templates.join(", ")
                    );
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
            let selected =
                Select::new("Choose a language", available_templates.to_vec()).prompt()?;
            Ok(TemplateSource::Local(selected))
        }
    }
}

pub async fn get_template_name(
    template_source: &TemplateSource,
    available_templates: &[String],
) -> anyhow::Result<(String, Option<TempDir>)> {
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
                    let options: Vec<String> = integrations
                        .options
                        .iter()
                        .map(|opt| format!("{} - {}", opt.name, opt.description))
                        .collect();

                    // Find the index of the default option
                    let default_index = integrations
                        .options
                        .iter()
                        .position(|opt| opt.name == integrations.default)
                        .unwrap_or(0);

                    let selected = Select::new("Choose build system integration", options)
                        .with_starting_cursor(default_index)
                        .prompt()?;
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

pub fn print_next_steps(project_name: &str, minimal_mode: bool) {
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

    println!();
    println!("🐛 Happy fuzzing!");
}
