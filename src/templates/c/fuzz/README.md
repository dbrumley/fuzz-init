# Fuzzing Setup for {{project_name}}

## Overview

This project uses universal fuzzing design - write once, fuzz everywhere! The same
`LLVMFuzzerTestOneInput()` function works with AFL, libFuzzer, HonggFuzz, and
standalone testing.

**Note**: This example demonstrates library-based integration. For adapting to your own project structure, see **Adapting to Your Project** section below and `INTEGRATION.md`.

## Targets

- `{{target_name}}`: Fuzz target that tests the GPS parser library functions

## Quick Start

{{#if (eq integration "make")}}
### Your Configuration: Makefile Integration with {{default_fuzzer}}

```bash
# 1. Build the library (from parent directory)
cd .. && make lib && cd fuzz

# 2. Build your selected fuzzer
make {{default_fuzzer}}

# 3. Run it
./{{target_name}}-{{default_fuzzer}} testcases/
{{#if (eq default_fuzzer "libfuzzer")}}
./{{target_name}}-{{default_fuzzer}} -dict=dictionaries/{{target_name}}.dict testcases/
{{/if}}
{{#if (eq default_fuzzer "afl")}}
mkdir -p findings
afl-fuzz -i testcases -o findings -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
mkdir -p corpus
honggfuzz -i testcases -W corpus -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
```

Or from the parent directory:
```bash
make lib && make fuzz    # Build library and fuzzer
```
{{/if}}

{{#if (eq integration "standalone")}}
### Your Configuration: Standalone Integration with {{default_fuzzer}}

```bash
# 1. Build the library (from parent directory)  
cd .. && make lib && cd fuzz

# 2. Build your selected fuzzer
./build.sh

# 3. Run it
./{{target_name}}-{{default_fuzzer}} testcases/
{{#if (eq default_fuzzer "libfuzzer")}}
./{{target_name}}-{{default_fuzzer}} -dict=dictionaries/{{target_name}}.dict testcases/
{{/if}}
{{#if (eq default_fuzzer "afl")}}
mkdir -p findings
afl-fuzz -i testcases -o findings -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
mkdir -p corpus
honggfuzz -i testcases -W corpus -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
```

Or from the parent directory:
```bash
make lib && make fuzz    # Build library and fuzzer
```
{{/if}}

{{#if (eq integration "cmake")}}
### Your Configuration: CMake Integration with {{default_fuzzer}}

```bash
# 1. Build the library (from parent directory)
cd .. && make lib && cd fuzz

# 2. Build your selected fuzzer
mkdir build && cd build
cmake ..
cmake --build . --target {{target_name}}-{{default_fuzzer}}

# 3. Run it
./{{target_name}}-{{default_fuzzer}} ../testcases/
{{#if (eq default_fuzzer "libfuzzer")}}
./{{target_name}}-{{default_fuzzer}} -dict=../dictionaries/{{target_name}}.dict ../testcases/
{{/if}}
{{#if (eq default_fuzzer "afl")}}
mkdir -p findings
afl-fuzz -i ../testcases -o findings -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
mkdir -p corpus  
honggfuzz -i ../testcases -W corpus -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
```
{{/if}}

### 2. Test Your Target

```bash
# Test with sample input
echo "FUZZ" | ./bin/{{target_name}}

# Test with files from testsuite
./bin/{{target_name}} testsuite/{{target_name}}/*
```

## Advanced Usage

### Build Other Fuzzer Types

{{#if (eq integration "make")}}
```bash
# Build additional fuzzer types (if engines are installed)
{{#if (eq default_fuzzer "standalone")}}
make afl            # Requires AFL++ installation
make libfuzzer      # Requires Clang with libFuzzer
make honggfuzz      # Requires HonggFuzz installation
{{/if}}
{{#if (eq default_fuzzer "afl")}}
make standalone     # No fuzzer engine required
make libfuzzer      # Requires Clang with libFuzzer
make honggfuzz      # Requires HonggFuzz installation
{{/if}}
{{#if (eq default_fuzzer "libfuzzer")}}
make standalone     # No fuzzer engine required
make afl            # Requires AFL++ installation
make honggfuzz      # Requires HonggFuzz installation
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
make standalone     # No fuzzer engine required
make afl            # Requires AFL++ installation
make libfuzzer      # Requires Clang with libFuzzer
{{/if}}
```
{{/if}}

{{#if (eq integration "standalone")}}
```bash
# Build with specific fuzzer types using environment variables
USE_AFL=1 ./build.sh           # AFL/AFL++ fuzzer
USE_LIBFUZZER=1 ./build.sh     # libFuzzer
USE_HONGGFUZZ=1 ./build.sh     # HonggFuzz
USE_STANDALONE=1 ./build.sh    # Standalone (no dependencies)
```
{{/if}}

{{#if (eq integration "cmake")}}
```bash
# Build different fuzzer types
cmake --build . --target {{target_name}}-standalone
cmake --build . --target {{target_name}}-afl  
cmake --build . --target {{target_name}}-libfuzzer
cmake --build . --target {{target_name}}-honggfuzz
```
{{/if}}

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

## Adapting to Your Project

This example uses a GPS parser library, but you can adapt it to your own project:

### Integration Approaches

**Library Linking (Recommended)**: If your project builds a static library, edit the Makefile to link against it:
```makefile
# In fuzz/Makefile
LIBS = -L../../lib -lmyproject
INCLUDES = -I../../include
```

**Direct Sources**: Include your source files directly:
```makefile
# In fuzz/Makefile  
PROJECT_SOURCES = ../../src/parser.c ../../src/utils.c
```

**Object Files**: Link against pre-built object files:
```makefile
# In fuzz/Makefile
PROJECT_OBJS = ../../build/parser.o ../../build/utils.o
```

### Quick Start for Your Project

1. **Read the detailed guide**: See `INTEGRATION.md` for step-by-step instructions
2. **Edit the Makefile**: Choose your integration approach and update paths
3. **Update the fuzz target**: Edit `src/{{target_name}}.c` with your function calls
4. **Test the build**: Run `make test` to verify everything works
5. **Add test cases**: Replace example inputs with relevant test data

### Common Adaptations

- **JSON parser**: Include `json_parse()` calls in your fuzz target
- **Network protocol**: Test packet parsing with protocol-specific inputs
- **File format**: Parse different file formats with appropriate test cases
- **Cryptographic functions**: Test key generation, encryption, validation

## Troubleshooting

### Build Failures

1. **Library not found**: If using the example, run `make lib` in parent directory first. If adapting to your project, see "Adapting to Your Project" above.
2. **AFL not found**: Install AFL++ or use `FUZZER_TYPE=standalone ./build.sh`
3. **libFuzzer not available**: Use a newer clang or try `FUZZER_TYPE=standalone ./build.sh`  
4. **HonggFuzz not found**: Install HonggFuzz or use a different fuzzer type

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
