# Template System Guide

This guide explains how to create and configure templates for fuzz-init. If you're adding support for a new language, this document will walk you through the process.

## Overview

The fuzz-init template system is designed to generate fuzzing harness scaffolds for different programming languages and build systems. Each template can support multiple build system integrations (Make, CMake, etc.) and two modes (full and minimal).

## Template Structure

Each language template lives in `src/templates/<LANGUAGE>/`. For example:
- C templates: `src/templates/C/`
- C++ templates: `src/templates/CPP/`
- Rust templates: `src/templates/Rust/`

Anything included under `src/templates` will be statically included in the
built binary so that `fuzz-init` is self-contained.  

### Directory Layout

A typical template directory structure:

```
src/templates/LANGUAGE/
├── template.toml           # Template metadata and configuration
├── fuzz/                   # Core fuzzing files (always included)
│   ├── src/
│   │   └── fuzz_harness_1.cpp  # Main fuzzing harness
│   ├── driver/
│   │   └── main.c          # Driver for non-libfuzzer
│   ├── Makefile            # Make integration
│   ├── CMakeLists.txt      # CMake integration
│   ├── cmake/              # CMake toolchain files
│   │   ├── libfuzzer.cmake 
│   │   ├── afl.cmake
│   │   ├── honggfuzz.cmake
│   │   └── standalone.cmake
│   ├── README.md           # Fuzzing-specific documentation
│   └── INTEGRATION.md      # Integration guide
├── src/                    # Example library code (full mode only)
│   ├── mylib.c
│   └── main.c
├── include/                # Header files (full mode only)
│   └── mylib.h
├── test/                   # Unit tests (full mode only)
│   └── test_lib.c/cpp/rs
├── Makefile                # Root Makefile (full mode only)
├── CMakeLists.txt          # Root CMake config (full mode only)
├── CMakePresets.json       # CMake presets (if using CMake)
├── fuzz.sh                 # Unified build script
├── TUTORIAL.md             # Complete fuzzing tutorial (full mode only)
└── Dockerfile              # Container setup (full mode only)
```

## Template Configuration (template.toml)

The `template.toml` file controls how your template behaves. Here's a complete example with explanations:

```toml
# Template metadata
[template]
name = "C"
description = "C fuzzing"
version = "1.0.0"

# Template variables with defaults
[variables]
project_name = { required = true, description = "Name of the fuzzing project" }
target_name = { default = "target1", description = "Name of the fuzz target" }

# Supported integration types
[integrations]
supported = ["make", "cmake"]
default = "cmake"

[[integrations.options]]
name = "make"
description = "Makefile-based build system integration"

[[integrations.options]]
name = "cmake"
description = "CMake-based build system integration"

# File conventions - smart defaults based on directory and extension
[file_conventions]
# Directories always included (core fuzz files)
always_include = ["fuzz"]

# Directories only in full mode (tutorial/example content)
full_mode_only = ["src", "include", "test", "test_data"]

# File extensions that should be templated
template_extensions = [".c", ".h", ".md", ".sh", ".txt"]

# File extensions that should not be templated
no_template_extensions = [".dict", ".bin"]

# Integration-specific files
[[files]]
condition = "integration == 'make'"
paths = ["fuzz/Makefile"]

[[files]]
condition = "integration == 'cmake'"
paths = [
    "fuzz/CMakeLists.txt",
    "fuzz/cmake/afl.cmake",
    "fuzz/cmake/honggfuzz.cmake", 
    "fuzz/cmake/libfuzzer.cmake",
    "fuzz/cmake/standalone.cmake"
]

# Root-level build files (full mode only)
[[files]]
condition = "integration == 'make' && minimal == false"
paths = ["Makefile"]

[[files]]
condition = "integration == 'cmake' && minimal == false"
paths = ["CMakeLists.txt", "CMakePresets.json"]

# Executable scripts
[[files]]
condition = "minimal == false"
paths = ["fuzz.sh"]
executable = true
template = true

# Dictionary file - uses template for filename but not content
[[files]]
path = "fuzz/dictionaries/{{target_name}}.dict"
template = true 

# Directory configurations
[[directories]]
path = "fuzz/testsuite/{{target_name}}"
create_empty = true  # Create empty directory for corpus files

# Post-generation message
[post_generation_message]
content = """
🎯 {{project_name}} C Fuzzing Project Created!

🚀 Quick Start:
{{#if minimal}}
1. cd {{project_name}}/fuzz
2. Edit src/fuzz_harness_1.c to fuzz your actual code
3. {{#if (eq integration 'make')}}make{{else if (eq integration 'cmake')}}cmake --preset fuzz-libfuzzer && cmake --build --preset fuzz-libfuzzer{{/if}}
{{else}}
1. cd {{project_name}}
2. {{#if (eq integration 'make')}}make && make fuzz{{else if (eq integration 'cmake')}}cmake --preset fuzz-libfuzzer && cmake --build --preset fuzz-libfuzzer{{/if}}
3. ./fuzz/build/fuzz_harness_1-libfuzzer fuzz/testsuite/
{{/if}}

Happy fuzzing! 🐛
"""

# Validation commands for testing
[validation]

[[validation.commands]]
name = "cmake-full"
condition = "integration == 'cmake' && minimal == false"
dir = "{{project_dir}}"
steps = [
    ["./fuzz.sh", "build"]
]
verify_files = [
    "build/libfuzzer/bin/fuzz_harness_1-libfuzzer",
    "build/afl/bin/fuzz_harness_1-afl",
    "build/honggfuzz/bin/fuzz_harness_1-honggfuzz",
    "build/standalone/bin/fuzz_harness_1-native"
]

[[validation.commands]]
name = "make-full"
condition = "integration == 'make' && minimal == false"
dir = "{{project_dir}}"
steps = [
    ["./fuzz.sh", "build"]
]
verify_files = [
    "fuzz/build/fuzz_harness_1-standalone"
]
```

## Templating System

The template system uses [Handlebars](https://handlebarsjs.com/) for templating. Both file contents AND filenames can use template variables.

### Available Variables

- `{{project_name}}` - The name of the project
- `{{target_name}}` - The name of the fuzz target (deprecated, use fixed names)
- `{{integration}}` - The selected integration type (make, cmake, etc.)
- `{{minimal}}` - Boolean indicating minimal mode

### Handlebars Helpers

- `{{#if condition}}...{{/if}}` - Conditional blocks
- `{{#unless condition}}...{{/unless}}` - Inverted conditionals
- `{{#if (eq var1 var2)}}...{{/if}}` - Equality comparison

### Example Usage

```c
// In fuzz_harness_1.c
{{#unless minimal}}
#include "lib.h"
{{else}}
// TODO: Replace this placeholder with your project's header files
// Example: #include "your_lib.h"
#include <stdio.h>  // For demonstration crash
{{/unless}}
```

## Key Design Principles

### 1. Fixed Harness Names

Use fixed names like `fuzz_harness_1.c` instead of `{{project_name}}.c` to avoid issues with special characters in project names.

### 2. Graceful Tool Detection

Your build system should detect available fuzzing tools and skip those that aren't installed rather than failing:

```makefile
# Detect available fuzzing tools
HAVE_CLANG := $(shell which clang >/dev/null 2>&1 && echo yes)
HAVE_AFL   := $(shell which afl-clang-fast >/dev/null 2>&1 && echo yes)
HAVE_HFUZZ := $(shell which hfuzz-clang >/dev/null 2>&1 && echo yes)

# Build only if tool is available
$(BUILD_DIR)/%-libfuzzer: src/%.c | $(BUILD_DIR)
	@if [ "$(HAVE_CLANG)" = "yes" ]; then \
	  echo "[libFuzzer] Building $@"; \
	  clang $(FLAGS) -fsanitize=fuzzer,address $< -o $@; \
	else \
	  echo "⏭️ libFuzzer skip (clang not found)"; \
	fi
```

### 3. Universal Fuzzing Interface

All fuzzers should implement the same interface:
```c
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
```

This allows users to switch between fuzzing engines without changing their code.

### 4. Minimal vs Full Mode

- **Full mode**: Complete example project with library, tests, and fuzzing setup
- **Minimal mode**: Just the fuzz directory, expecting users to integrate with their existing project

### 5. Build System Independence

Provide multiple integration options (Make, CMake, standalone build.sh) so users can choose what fits their project.

## Creating a New Language Template

### Step 1: Create Directory Structure

```bash
mkdir -p src/templates/MYLANG/fuzz/{src,driver,cmake,dictionaries,testsuite}
```

### Step 2: Create template.toml

Start with the example above and modify for your language:
- Update language name and description
- Adjust file extensions in `template_extensions`
- Define integration-specific files

### Step 3: Create Core Files

#### fuzz/src/fuzz_harness_1.{ext}
The main fuzzing harness. Should implement `LLVMFuzzerTestOneInput`.

#### fuzz/driver/main.{ext}
Universal driver for AFL++, HonggFuzz, and standalone fuzzing.

#### fuzz/Makefile and/or fuzz/CMakeLists.txt
Build configuration with graceful tool detection.

### Step 4: Create Example Code (Full Mode)

- `src/lib.{ext}` - Example library with intentional bugs
- `include/lib.{h/hpp}` - Header files
- `test/test_lib.{ext}` - Unit tests

### Step 5: Create Documentation

- `fuzz/README.md` - Quick reference for fuzzing
- `fuzz/INTEGRATION.md` - How to integrate with existing projects
- `TUTORIAL.md` - Complete fuzzing tutorial (full mode only)

### Step 6: Create Build Scripts

#### fuzz.sh
Unified script that builds all fuzzer variants:
```bash
#!/usr/bin/env bash
set -euo pipefail

ENGINES=("libfuzzer" "afl" "honggfuzz" "standalone")

build_engine() {
  local engine="$1"
  {{#if (eq integration 'cmake')}}
  cmake --preset "fuzz-$engine" && cmake --build --preset "fuzz-$engine"
  {{else if (eq integration 'make')}}
  make "fuzz-$engine"
  {{/if}}
}

build_all() {
  for engine in "${ENGINES[@]}"; do
    build_engine "$engine" || true
  done
}
```

### Step 7: Add Validation Commands

In template.toml, add validation commands that test your template:
```toml
[[validation.commands]]
name = "mylang-make-full"
condition = "integration == 'make' && minimal == false"
steps = [["./fuzz.sh", "build"]]
verify_files = ["fuzz/build/fuzz_harness_1-standalone"]
```

## Testing Your Template

Use the dev mode to test your template:

```bash
cargo run -- --dev-mode --language MYLANG --dev-output ./test-output
```

This will:
1. Generate projects with all combinations of integrations and modes
2. Run validation commands
3. Verify expected files are created
4. Report any failures

## Best Practices

1. **Start Simple**: Begin with one integration type (e.g., Make) and get it working before adding others.

2. **Copy and Adapt**: Look at existing templates (C and CPP) for patterns and conventions.

3. **Test Edge Cases**: Ensure your template works with:
   - Project names containing spaces or special characters
   - Missing fuzzing tools
   - Both full and minimal modes

4. **Document Thoroughly**: Include clear instructions in README.md and INTEGRATION.md.

5. **Follow Conventions**: 
   - Use consistent file naming (fuzz_harness_1, not project-specific names)
   - Place binaries in predictable locations
   - Support all four fuzzer types when possible

6. **Provide Examples**: Include example bugs in full mode to help users understand fuzzing.

## Common Pitfalls

1. **Hardcoded Paths**: Use relative paths and template variables instead.

2. **Missing Tool Detection**: Always check if fuzzing tools exist before using them.

3. **Sanitizer Conflicts**: Be careful when mixing code compiled with different sanitizers.

4. **Template Variable Issues**: Test with various project names to ensure templating works correctly.

## Getting Help

- Check existing templates in `src/templates/` for examples
- Run templates in dev mode to debug issues
- Look at validation output for specific error messages

Remember: The goal is to make fuzzing as easy as possible for users of your language!