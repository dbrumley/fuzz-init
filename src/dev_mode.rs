use crate::cli::Args;
use crate::template_processor::*;
use crate::types::*;
use anyhow::{anyhow, Result};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct TestConfiguration {
    pub language: String,
    pub integration: String,
    pub minimal: bool,
}

#[derive(Debug)]
pub struct TestResult {
    pub config: TestConfiguration,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub build_log: String,
}

#[derive(Debug)]
pub struct DevSession {
    pub workspace_dir: PathBuf,
    pub temp_dir: Option<TempDir>,
    pub results: Vec<TestResult>,
    pub start_time: Instant,
}

impl DevSession {
    pub fn new(custom_output: Option<&str>) -> Result<Self> {
        let (workspace_dir, temp_dir) = if let Some(output_path) = custom_output {
            let path = PathBuf::from(output_path);
            std::fs::create_dir_all(&path)?;
            (path, None)
        } else {
            let temp = TempDir::new()?;
            let path = temp.path().to_path_buf();
            (path, Some(temp))
        };

        Ok(DevSession {
            workspace_dir,
            temp_dir,
            results: Vec::new(),
            start_time: Instant::now(),
        })
    }

    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result);
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
        self.start_time = Instant::now();
    }
}

pub async fn run_dev_mode(args: &Args) -> Result<()> {
    // Validate dev-mode arguments
    validate_dev_mode_args(args)?;

    // Create development session
    let mut session = DevSession::new(args.dev_output.as_deref())?;

    println!("🔧 Starting template development mode...");
    println!("📁 Workspace: {}", session.workspace_dir.display());

    // Run initial validation
    run_template_validation(args, &mut session).await?;

    // If watch mode is enabled, start file watching
    if let Some(watch_path) = &args.watch {
        // If watch path is "auto" and we have a language, use the template directory
        let actual_watch_path = if watch_path == "auto" && args.language.is_some() {
            format!("src/templates/{}/", args.language.as_ref().unwrap())
        } else if watch_path == "auto" && args.language.is_none() {
            return Err(anyhow!(
                "--watch without path requires --language to be specified"
            ));
        } else {
            watch_path.clone()
        };

        println!("\n👀 Watching {} for changes...", actual_watch_path);
        println!("Press Ctrl+C to exit");

        start_file_watcher(&actual_watch_path, args, session).await?;
    } else {
        // Just run once and exit
        print_final_report(&session);
    }

    Ok(())
}

fn validate_dev_mode_args(args: &Args) -> Result<()> {
    if !args.dev_mode {
        return Err(anyhow!("This function should only be called in dev-mode"));
    }

    // Watch requires dev-mode
    if args.watch.is_some() && !args.dev_mode {
        return Err(anyhow!("--watch requires --dev-mode"));
    }

    // Must specify either language or template for dev-mode
    if args.language.is_none() && args.template.is_none() {
        return Err(anyhow!(
            "--dev-mode requires either --language or --template"
        ));
    }

    // Cannot specify both project name and dev-mode (we use temp names)
    if args.project_name_pos.is_some() || args.project.is_some() {
        return Err(anyhow!(
            "--dev-mode generates temporary project names. Remove project name argument."
        ));
    }

    Ok(())
}

async fn run_template_validation(args: &Args, session: &mut DevSession) -> Result<()> {
    let start_time = Instant::now();

    // Get available templates
    let _available_templates = get_available_templates()?;

    // Determine which language to test
    let language = if let Some(lang) = &args.language {
        lang.clone()
    } else if let Some(_template) = &args.template {
        // For remote templates, use a generic name
        "remote".to_string()
    } else {
        return Err(anyhow!("Must specify --language or --template"));
    };

    // Load template metadata
    let metadata = load_template_metadata(&language)?;

    // Generate test configurations
    let configs = generate_test_configurations(&language, args, metadata.as_ref())?;

    println!(
        "\n🧪 Testing {} configurations for {} template:",
        configs.len(),
        language
    );

    // Run tests
    for (i, config) in configs.iter().enumerate() {
        println!(
            "\n[{}/{}] Testing: {}  + {}",
            i + 1,
            configs.len(),
            //config.fuzzer,
            config.integration,
            if config.minimal { "minimal" } else { "full" }
        );

        let result = test_configuration(
            config,
            session,
            metadata.as_ref(),
            session.temp_dir.is_none(),
        )
        .await;
        match &result {
            Ok(test_result) => {
                let status = if test_result.success { "✅" } else { "❌" };
                println!(
                    "    {} {} ({:.1}s)",
                    status,
                    format_config_name(config),
                    test_result.duration.as_secs_f32()
                );
                if !test_result.success
                    && !test_result
                        .error
                        .as_ref()
                        .unwrap_or(&String::new())
                        .is_empty()
                {
                    println!("       Error: {}", test_result.error.as_ref().unwrap());
                }
            }
            Err(e) => {
                println!("    ❌ Failed to run test: {}", e);
            }
        }

        if let Ok(test_result) = result {
            session.add_result(test_result);
        }
    }

    let total_duration = start_time.elapsed();
    println!(
        "\n⏱️  Total validation time: {:.1}s",
        total_duration.as_secs_f32()
    );

    // Print summary
    print_results_summary(&session.results);

    Ok(())
}

fn generate_test_configurations(
    language: &str,
    args: &Args,
    metadata: Option<&TemplateMetadata>,
) -> Result<Vec<TestConfiguration>> {
    let mut configs = Vec::new();

    // Get supported configurations from metadata or defaults
    let supported_integrations = if let Some(meta) = metadata {
        let integrations = meta
            .integrations
            .as_ref()
            .map(|i| i.supported.clone())
            .unwrap_or_else(|| vec!["script".to_string()]);
        integrations
    } else {
        vec!["script".to_string()]
    };

    // Filter by user-specified options
    /* let fuzzers_to_test = if let Some(fuzzer) = &args.fuzzer {
        vec![fuzzer.clone()]
    } else {
        supported_fuzzers
    }; */

    let integrations_to_test = if let Some(integration) = &args.integration {
        vec![integration.clone()]
    } else {
        supported_integrations
    };

    let modes_to_test = if args.minimal {
        vec![true] // Only minimal
    } else {
        vec![false, true] // Both full and minimal
    };

    // Generate all combinations
    /* for fuzzer in &fuzzers_to_test { */
    for integration in &integrations_to_test {
        for &minimal in &modes_to_test {
            configs.push(TestConfiguration {
                language: language.to_string(),
                //fuzzer: fuzzer.clone(),
                integration: integration.clone(),
                minimal,
            });
        }
    }
    /*  } */

    Ok(configs)
}

async fn test_configuration(
    config: &TestConfiguration,
    session: &DevSession,
    metadata: Option<&TemplateMetadata>,
    preserve_projects: bool,
) -> Result<TestResult> {
    let start_time = Instant::now();

    // Generate unique project name for this test
    let project_name = format!(
        "test-{}-{}-{}",
        config.language,
        //config.fuzzer,
        config.integration,
        if config.minimal { "min" } else { "full" }
    );

    let project_dir = session.workspace_dir.join(&project_name);

    // Clean up any existing project directory
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir)?;
    }

    // Generate project using template system
    let mut build_log = String::new();
    let mut success = false;
    let mut error_msg = None;

    match generate_test_project(&project_dir, config, metadata).await {
        Ok(_) => {
            // Try to build the project
            match build_test_project(&project_dir, config, &mut build_log).await {
                Ok(_) => {
                    success = true;
                    build_log.push_str("\n✅ Build successful");
                }
                Err(e) => {
                    error_msg = Some(format!("Build failed: {}", e));
                    build_log.push_str(&format!("\n❌ Build failed: {}", e));
                }
            }
        }
        Err(e) => {
            error_msg = Some(format!("Template generation failed: {}", e));
            build_log.push_str(&format!("❌ Template generation failed: {}", e));
        }
    }

    // Clean up project directory (keep logs) unless preserving for debugging
    if project_dir.exists() && !preserve_projects {
        let _ = std::fs::remove_dir_all(&project_dir);
    }

    let duration = start_time.elapsed();

    Ok(TestResult {
        config: config.clone(),
        success,
        duration,
        error: error_msg,
        build_log,
    })
}

async fn generate_test_project(
    project_dir: &Path,
    config: &TestConfiguration,
    metadata: Option<&TemplateMetadata>,
) -> Result<()> {
    // Set up handlebars and data for template processing
    let handlebars = setup_handlebars();

    let data = json!({
        "project_name": project_dir.file_name().unwrap().to_str().unwrap(),
        "target_name": project_dir.file_name().unwrap().to_str().unwrap(),
        //"default_fuzzer": config.fuzzer,
        "integration": config.integration,
        "minimal": config.minimal
    });

    // Process template
    process_template_directory(&config.language, project_dir, &handlebars, &data, metadata)?;

    Ok(())
}

// TODO: Replace this with calling a template-defined command/script.
async fn build_test_project(
    project_dir: &Path,
    config: &TestConfiguration,
    build_log: &mut String,
) -> Result<()> {
    match config.integration.as_str() {
        "standalone" => build_standalone_project(project_dir, config, build_log).await,
        "make" => build_makefile_project(project_dir, config, build_log).await,
        "cmake" => build_cmake_project(project_dir, config, build_log).await,
        _ => Err(anyhow!(
            "Unsupported integration type: {}",
            config.integration
        )),
    }
}

async fn build_standalone_project(
    project_dir: &Path,
    config: &TestConfiguration,
    build_log: &mut String,
) -> Result<()> {
    let build_script = if config.minimal {
        project_dir.join("fuzz").join("build.sh")
    } else {
        project_dir.join("build.sh")
    };

    if !build_script.exists() {
        return Err(anyhow!(
            "Build script not found: {}",
            build_script.display()
        ));
    }

    build_log.push_str(&format!("Running: {}\n", build_script.display()));

    let output = Command::new("bash")
        .arg(&build_script)
        .current_dir(if config.minimal {
            project_dir.join("fuzz")
        } else {
            project_dir.to_path_buf()
        })
        .output()?;

    build_log.push_str(&format!(
        "stdout: {}\n",
        String::from_utf8_lossy(&output.stdout)
    ));
    build_log.push_str(&format!(
        "stderr: {}\n",
        String::from_utf8_lossy(&output.stderr)
    ));

    if !output.status.success() {
        return Err(anyhow!(
            "Build script failed with exit code: {:?}",
            output.status.code()
        ));
    }

    Ok(())
}

async fn build_makefile_project(
    project_dir: &Path,
    config: &TestConfiguration,
    build_log: &mut String,
) -> Result<()> {
    let make_dir = if config.minimal {
        project_dir.join("fuzz")
    } else {
        project_dir.to_path_buf()
    };

    build_log.push_str(&format!("Running: make in {}\n", make_dir.display()));

    let output = Command::new("make").current_dir(&make_dir).output()?;

    build_log.push_str(&format!(
        "stdout: {}\n",
        String::from_utf8_lossy(&output.stdout)
    ));
    build_log.push_str(&format!(
        "stderr: {}\n",
        String::from_utf8_lossy(&output.stderr)
    ));

    if !output.status.success() {
        return Err(anyhow!(
            "Make failed with exit code: {:?}",
            output.status.code()
        ));
    }

    Ok(())
}

async fn build_cmake_project(
    project_dir: &Path,
    config: &TestConfiguration,
    build_log: &mut String,
) -> Result<()> {
    let cmake_dir = if config.minimal {
        project_dir.join("fuzz")
    } else {
        project_dir.to_path_buf()
    };

    let build_dir = cmake_dir.join("build");
    std::fs::create_dir_all(&build_dir)?;

    build_log.push_str(&format!("Running: cmake .. in {}\n", build_dir.display()));

    // Configure
    let output = Command::new("cmake")
        .arg("..")
        .current_dir(&build_dir)
        .output()?;

    build_log.push_str(&format!(
        "cmake stdout: {}\n",
        String::from_utf8_lossy(&output.stdout)
    ));
    build_log.push_str(&format!(
        "cmake stderr: {}\n",
        String::from_utf8_lossy(&output.stderr)
    ));

    if !output.status.success() {
        return Err(anyhow!(
            "CMake configure failed with exit code: {:?}",
            output.status.code()
        ));
    }

    // Build
    let output = Command::new("cmake")
        .args(&["--build", "."])
        .current_dir(&build_dir)
        .output()?;

    build_log.push_str(&format!(
        "cmake build stdout: {}\n",
        String::from_utf8_lossy(&output.stdout)
    ));
    build_log.push_str(&format!(
        "cmake build stderr: {}\n",
        String::from_utf8_lossy(&output.stderr)
    ));

    if !output.status.success() {
        return Err(anyhow!(
            "CMake build failed with exit code: {:?}",
            output.status.code()
        ));
    }

    Ok(())
}

async fn start_file_watcher(watch_path: &str, args: &Args, mut session: DevSession) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    watcher.watch(Path::new(watch_path), RecursiveMode::Recursive)?;

    let mut last_rebuild = Instant::now();
    let debounce_duration = Duration::from_millis(500);

    loop {
        match rx.recv_timeout(debounce_duration) {
            Ok(_event) => {
                // Check if enough time has passed since last rebuild
                if last_rebuild.elapsed() >= debounce_duration {
                    last_rebuild = Instant::now();

                    // Clear terminal and show rebuild message
                    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
                    println!("🔄 Template changed, rebuilding...\n");

                    // Clear previous results
                    session.clear_results();

                    // Re-run validation
                    if let Err(e) = run_template_validation(args, &mut session).await {
                        println!("❌ Validation failed: {}", e);
                    }

                    println!("\n👀 Watching for changes... (Press Ctrl+C to exit)");
                }
            }
            Err(_) => {
                // Timeout - continue watching
                continue;
            }
        }
    }
}

fn print_results_summary(results: &[TestResult]) {
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total - successful;

    println!("\n═══════════════════════════════════════");
    println!("📊 Test Results Summary");
    println!("═══════════════════════════════════════");
    println!("Total:      {}", total);
    println!("✅ Success: {}", successful);
    println!("❌ Failed:  {}", failed);

    if failed > 0 {
        println!("\n❌ Failed configurations:");
        for result in results.iter().filter(|r| !r.success) {
            println!(
                "  • {} - {}",
                format_config_name(&result.config),
                result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }
    }

    let avg_duration = results
        .iter()
        .map(|r| r.duration.as_secs_f32())
        .sum::<f32>()
        / total as f32;

    println!("\n⏱️  Average build time: {:.1}s", avg_duration);

    let success_rate = (successful as f32 / total as f32) * 100.0;
    println!("📈 Success rate: {:.1}%", success_rate);
}

fn print_final_report(session: &DevSession) {
    let total_time = session.start_time.elapsed();

    println!("\n🎯 Development Session Complete");
    println!("═══════════════════════════════════════");
    println!("📁 Workspace: {}", session.workspace_dir.display());
    println!("⏱️  Total time: {:.1}s", total_time.as_secs_f32());

    print_results_summary(&session.results);
}

fn format_config_name(config: &TestConfiguration) -> String {
    format!(
        "{}  + {}",
        //config.fuzzer,
        config.integration,
        if config.minimal { "minimal" } else { "full" }
    )
}
