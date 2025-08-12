use clap::Parser;
use serde_json::json;
use std::path::Path;

mod cli;
mod dev_mode;
mod github_fetcher;
mod template_processor;
mod types;

// use types::*; // Not needed in main
use cli::*;
use template_processor::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Check if documentation generation was requested
    if args.generate_docs {
        generate_cli_documentation();
        return Ok(());
    }

    // Check if dev mode was requested
    if args.dev_mode {
        return dev_mode::run_dev_mode(&args).await;
    }

    // Get available templates
    let available_templates = get_available_templates()?;
    if available_templates.is_empty() {
        anyhow::bail!("No embedded templates found.");
    }

    // Track which values came from prompts for hint generation
    let mut prompted_values = types::PromptedValues::default();

    // Get all necessary inputs
    let (project_name, prompted_project) = get_project_name_with_tracking(&args)?;
    prompted_values.project_name = prompted_project;

    let (template_source, prompted_language) =
        determine_template_source_with_tracking(&args, &available_templates)?;
    prompted_values.language = prompted_language;

    let (template_name, template_path) =
        get_template_name(&template_source, &available_templates).await?;

    // Load template metadata based on template type
    let metadata = if let Some(ref path) = template_path {
        load_template_metadata_from_path(path)?
    } else {
        load_template_metadata(&template_name)?
    };

    let (integration_type, prompted_integration) =
        select_integration_with_tracking(&args, metadata.as_ref())?;
    prompted_values.integration = prompted_integration;

    let minimal_mode = args.minimal;

    // Setup Handlebars with helpers
    let handlebars = setup_handlebars();

    // Extract base name for filenames (remove path components)
    let project_basename = Path::new(&project_name)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let data = json!({
        "project_name": project_name,
        "target_name": project_basename, // Use base name only for template filenames
        "integration": integration_type,
        "minimal": minimal_mode
    });

    // Generate project - handle nested paths properly
    let out_path = Path::new(&project_name);

    // Create parent directories if they don't exist
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() && parent != Path::new(".") {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create parent directories for '{}': {}",
                    project_name,
                    e
                )
            })?;
        }
    }

    // Process template based on type
    if let Some(ref path) = template_path {
        // Remote template - process from filesystem
        process_filesystem_template_directory(
            path,
            out_path,
            &handlebars,
            &data,
            metadata.as_ref(),
        )?;
    } else {
        // Embedded template - process from embedded resources
        process_template_directory(
            &template_name,
            out_path,
            &handlebars,
            &data,
            metadata.as_ref(),
        )?;
    }

    // Success message with next steps
    println!("Project '{project_name}' created with {template_name} template!");

    // Display post-generation message if it exists
    display_post_generation_message(out_path)?;

    print_next_steps(
        &project_name,
        minimal_mode,
        &prompted_values,
        &template_source,
        &integration_type,
    );

    Ok(())
}

fn generate_cli_documentation() {
    let markdown = clap_markdown::help_markdown::<Args>();

    // Convert to MDX with Docusaurus enhancements
    #[allow(clippy::uninlined_format_args)]
    let mdx_content = format!(
        r#"---
title: "fuzz-init CLI Reference"
description: "Complete command-line reference for fuzz-init, a tool for scaffolding fuzzing projects"
sidebar_label: "CLI Reference"
sidebar_position: 1
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import CodeBlock from '@theme/CodeBlock';

# fuzz-init CLI Reference

The `fuzz-init` command-line tool helps you scaffold fuzzing projects for multiple programming languages with comprehensive fuzzing infrastructure.

## Quick Start Examples

<Tabs>
<TabItem value="c" label="C Project">

```bash
# Full C project with tutorial
fuzz-init my-c-project --language c --fuzzer libfuzzer --integration make

# Minimal C fuzzing setup for existing projects  
fuzz-init my-c-project --language c --minimal --fuzzer libfuzzer --integration make
```

</TabItem>
<TabItem value="cpp" label="C++ Project">

```bash
# Full C++ project with comprehensive setup
fuzz-init my-cpp-project --language cpp --fuzzer afl --integration make

# C++ with CMake integration
fuzz-init my-cpp-project --language cpp --fuzzer libfuzzer --integration cmake
```

</TabItem>
<TabItem value="rust" label="Rust Project">

```bash
# Rust project with cargo-fuzz
fuzz-init my-rust-project --language rust --fuzzer libfuzzer

# Minimal Rust fuzzing setup
fuzz-init my-rust-project --language rust --minimal
```

</TabItem>
<TabItem value="python" label="Python Project">

```bash
# Python fuzzing project
fuzz-init my-python-project --language python

# Python with specific setup
fuzz-init my-python-project --language python --minimal
```

</TabItem>
</Tabs>

## Installation

:::tip Prerequisites
Make sure you have the required tools installed:
- **Rust toolchain** - For building fuzz-init
- **clang/clang++** - For libFuzzer support (C/C++ templates)
- **AFL/AFL++** - Optional, for AFL fuzzing mode
- **HonggFuzz** - Optional, for HonggFuzz mode
:::

### From Source

```bash
git clone https://github.com/forallsecure/fuzz-init
cd fuzz-init
cargo build --release
./target/release/fuzz-init --help
```

## Usage Patterns

### Interactive Mode
Run without arguments to be prompted for all options:

```bash
fuzz-init
```

### Full Command Specification
Specify all options for non-interactive usage:

```bash
fuzz-init PROJECT_NAME --language LANG --fuzzer FUZZER --integration TYPE [--minimal]
```

### Remote Templates
Use templates from GitHub repositories:

```bash
fuzz-init my-project --template github:org/repo
fuzz-init my-project --template @org/repo  # Short syntax
```

## Integration Types

<Tabs>
<TabItem value="standalone" label="Standalone">

**Best for**: New projects or standalone fuzzing setups

- Self-contained build scripts
- No external build system dependencies
- Works out of the box

```bash
fuzz-init my-project --language c --integration standalone
```

</TabItem>
<TabItem value="makefile" label="Makefile">

**Best for**: Projects already using Make

- Generates Makefile with fuzzer targets
- Integrates with existing Make-based builds
- Supports `make fuzz-libfuzzer`, `make fuzz-afl`, etc.

```bash
fuzz-init my-project --language c --integration makefile
```

</TabItem>
<TabItem value="cmake" label="CMake">

**Best for**: Projects already using CMake

- Generates CMakeLists.txt with fuzzer targets
- Integrates with existing CMake builds  
- Supports `cmake --build . --target fuzz-libfuzzer`

```bash
fuzz-init my-project --language c --integration cmake
```

</TabItem>
</Tabs>

## Template Modes

### Full Mode (Default)
Creates a complete project with:
- Example application code
- Comprehensive tutorials and documentation
- Unit tests and integration examples
- Docker and CI/CD configurations

:::tip Learning Fuzzing
Use full mode when learning fuzzing or starting from scratch. It includes extensive tutorials and examples.
:::

### Minimal Mode
Creates just the fuzzing infrastructure:
- Essential fuzzing files only
- Integration documentation
- Ready for existing project integration

:::warning Existing Projects
Use minimal mode (`--minimal`) when adding fuzzing to existing codebases. It creates only the necessary fuzzing files.
:::

## Command Reference

{}

## Advanced Examples

### Multi-Fuzzer Setup
Generate a project that works with multiple fuzzers:

```bash
# Generate with libFuzzer as default, but supports AFL and HonggFuzz
fuzz-init multi-fuzzer --language c --fuzzer libfuzzer --integration make
cd multi-fuzzer
make fuzz-libfuzzer  # Build for libFuzzer
make fuzz-afl        # Build for AFL
```

### Custom Template Integration
Use your own fuzzing templates:

```bash
# Use organization template
fuzz-init my-project --template @myorg/custom-c-template

# Use specific repository with subdirectory
fuzz-init my-project --template github:myorg/templates/c-advanced
```

### Testing Template Compatibility
Verify templates work on your system:

```bash
# Test all templates and fuzzers
fuzz-init --test

# This will show which combinations work on your system
```

## Template-Specific Features

### C Template
- **Universal fuzzing design**: Write `LLVMFuzzerTestOneInput` once, works with all fuzzers
- **Library-based architecture**: Builds proper libraries that fuzzing harnesses link against  
- **Comprehensive testing**: Unit tests, integration tests, and fuzzing workflows
- **Multi-platform support**: Works on macOS, Linux with or without fuzzer tools installed

### C++ Template  
- **AFL driver support**: Includes AFL-compatible driver for C++ projects
- **Modern C++**: Uses current C++ standards and best practices
- **Build system integration**: Full Makefile and build script support

### Rust Template
- **cargo-fuzz integration**: Native Rust fuzzing with cargo-fuzz
- **AFL.rs support**: AFL fuzzing for Rust projects
- **Cargo workspace**: Proper Rust project structure with fuzzing workspace

### Python Template
- **Basic structure**: Simple Python project scaffolding
- **Extensible**: Foundation for Python-specific fuzzing tools

## Troubleshooting

:::warning Common Issues
**AFL not working**: Make sure AFL++ is properly installed and in your PATH
**libFuzzer not found**: Ensure you have clang installed with libFuzzer support
**Build failures**: Run `fuzz-init --test` to identify missing dependencies
:::

### Template Validation
Use the test mode to validate templates work on your system:

```bash
fuzz-init --test
```

This will:
- Test all available templates
- Try building with every fuzzer option
- Show which combinations work
- Identify missing dependencies

## See Also

- [C Fuzzing Tutorial](/docs/c/intro) - Complete guide to C fuzzing
- [Rust Fuzzing Tutorial](/docs/rust/intro) - Rust-specific fuzzing techniques  
- [Fuzzing Fundamentals](/docs/fundamentals/what-is-fuzzing) - Introduction to fuzzing concepts
- [Mayhem Platform](/docs/mayhem/installation) - Advanced fuzzing with Mayhem Security
"#,
        markdown
    );

    println!("{mdx_content}");
}

/// Display post-generation message if it exists and then clean it up
fn display_post_generation_message(output_dir: &Path) -> anyhow::Result<()> {
    let message_path = output_dir.join("POST_GENERATION_MESSAGE.md");

    if message_path.exists() {
        // Read and display the message
        let message_content = std::fs::read_to_string(&message_path)?;

        println!("\n{message_content}");

        // Clean up the message file after displaying it
        std::fs::remove_file(&message_path)?;
    }

    Ok(())
}
