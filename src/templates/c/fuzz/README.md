# Fuzzing Setup for {{project_name}}

## Overview

This project uses universal fuzzing design - write once, fuzz everywhere! The same
`LLVMFuzzerTestOneInput()` function works with AFL, libFuzzer, HonggFuzz, and
standalone testing.

## Targets

- `{{target_name}}`: Example fuzz target that triggers on "FUZZ" input

## Quick Start

### 1. Build (Default Fuzzer)

```bash
cd fuzz
./build.sh
```

### 2. Test Your Target

```bash
# Test with sample input
echo "FUZZ" | ./bin/{{target_name}}

# Test with files from testsuite
./bin/{{target_name}} testsuite/{{target_name}}/*
```

## Advanced Usage

### Build with Specific Fuzzer

```bash
# AFL/AFL++ (requires AFL installation)
FUZZER_TYPE=afl ./build.sh

# libFuzzer (requires clang with libFuzzer support)  
FUZZER_TYPE=libfuzzer ./build.sh

# HonggFuzz (requires HonggFuzz installation)
FUZZER_TYPE=hongfuzz ./build.sh

# Standalone (no dependencies)
FUZZER_TYPE=standalone ./build.sh
```

### Running Different Fuzzer Types

#### AFL/AFL++
```bash
mkdir -p input output
echo 'test' > input/test.txt
afl-fuzz -i input -o output ./bin/{{target_name}}
```

#### libFuzzer
```bash
./bin/{{target_name}} testsuite/{{target_name}}/ -dict=dictionaries/{{target_name}}.dict
```

#### HonggFuzz
```bash
mkdir -p input
echo 'test' > input/test.txt
honggfuzz -i input -- ./bin/{{target_name}}
```

#### Standalone Testing
```bash
# Test with stdin
echo 'FUZZ' | ./bin/{{target_name}}

# Test with files
./bin/{{target_name}} testsuite/{{target_name}}/*

# Test multiple files
for file in testsuite/{{target_name}}/*; do
    echo "Testing $file"
    ./bin/{{target_name}} "$file"
done
```

## Customizing Your Fuzz Target

Edit `src/{{target_name}}.c` and modify the `LLVMFuzzerTestOneInput()` function:

```c
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Add your fuzzing logic here
    // This function will be called with random/mutated input
    
    if (size > 4 && data[0] == 'F' && data[1] == 'U' && data[2] == 'Z' && data[3] == 'Z') {
        printf("Boom!\n");  // This will be caught by fuzzers as interesting
    }
    
    return 0;  // Always return 0 for successful execution
}
```

## Project Structure

```
fuzz/
├── src/{{target_name}}.c          # Your fuzz target implementation
├── driver/main.c                  # Universal fuzzer driver (don't modify)
├── build.sh                       # Smart build script
├── dictionaries/{{target_name}}.dict  # Fuzzing dictionary
├── testsuite/{{target_name}}/     # Test cases and corpus
├── Dockerfile                     # For containerized fuzzing  
├── Mayhemfile                     # Mayhem configuration
└── README.md                      # This file
```

## Using with Mayhem

For cloud-based fuzzing with Mayhem:

```bash
mayhem run
```

## Troubleshooting

### Build Failures

1. **AFL not found**: Install AFL++ or use `FUZZER_TYPE=standalone ./build.sh`
2. **libFuzzer not available**: Use a newer clang or try `FUZZER_TYPE=standalone ./build.sh`  
3. **HonggFuzz not found**: Install HonggFuzz or use a different fuzzer type

### Getting Help

```bash
./build.sh --help    # Show all fuzzer options and examples
```

### Platform Issues

#### macOS
- **AFL compatibility**: AFL++ may have library dependency issues on Apple Silicon (ARM64)
- **Recommended**: Use `FUZZER_TYPE=libfuzzer` or `FUZZER_TYPE=standalone` on macOS
- **Alternative**: Use the provided devcontainer for full Linux compatibility

#### Linux
- **All fuzzer types work**: AFL++, libFuzzer, HonggFuzz, and standalone all function properly
- **Installation**: 
  - AFL++: `sudo apt install afl++` or build from source
  - HonggFuzz: `sudo apt install honggfuzz` or build from source
  - libFuzzer: Included with modern clang installations

#### Windows
- **WSL recommended**: Use Windows Subsystem for Linux for best compatibility
- **Native support**: Limited, use containerized approach

#### Containers/CI
- **Docker**: Use the provided Dockerfile for consistent cross-platform results
- **Devcontainer**: VS Code devcontainer available with all tools pre-installed
- **CI/CD**: Container approach ensures reproducible builds across environments

### Development Environment

For the best development experience with all fuzzing tools working:

1. **VS Code Devcontainer** (Recommended):
   - All fuzzing tools pre-installed and configured
   - No platform-specific issues
   - Consistent Linux environment

2. **Native Linux**:
   - Install fuzzing tools via package manager
   - All features work out-of-the-box

3. **macOS**:
   - libFuzzer and standalone work reliably
   - AFL++ may require troubleshooting or alternative installation methods
