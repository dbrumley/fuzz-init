# Integrating {{project_name}} Fuzzing with Your Build System

This directory contains fuzzing infrastructure for {{project_name}} that can be 
integrated with your existing build system.

## Integration Overview

The fuzzing setup is designed to work alongside your existing project without 
modifying your main build system. All fuzzing-specific files are contained in 
this `fuzz/` directory.

## Directory Structure

```
fuzz/
├── src/{{target_name}}.c       # Your fuzz target implementation
├── driver/main.c               # Universal fuzzer driver
├── testcases/                  # Initial test inputs
├── dictionaries/               # Fuzzing dictionaries
├── Makefile                    # Makefile integration (if selected)
├── CMakeLists.txt             # CMake integration (if selected)
├── build.sh                   # Standalone build script
├── Mayhemfile                 # Mayhem configuration
├── Dockerfile                 # Container for fuzzing
└── INTEGRATION.md             # This file
```

## Implementation Steps

### 1. Implement Your Fuzz Target

Edit `src/{{target_name}}.c` and implement your fuzzing logic in the 
`LLVMFuzzerTestOneInput` function:

```c
#include <stdint.h>
#include <stddef.h>
// Include your project headers here
#include "your_project.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Your fuzzing logic here
    // Call functions from your main project
    your_function(data, size);
    return 0;
}
```

### 2. Update Source Paths

{{#if (eq integration "makefile")}}
#### Makefile Integration

Edit the `Makefile` to point to your project's source files:

```makefile
# Update these paths to match your project structure
SRC_DIR = ../src                    # Path to your source files
INCLUDES = -I$(SRC_DIR) -I../include   # Include directories
```

Add any additional source files your fuzz target needs:

```makefile
# Add your project's source files
PROJECT_SOURCES = $(SRC_DIR)/module1.c $(SRC_DIR)/module2.c
FUZZ_SRC = $(SRC_DIR)/$(TARGET).c $(PROJECT_SOURCES)
```
{{/if}}

{{#if (eq integration "cmake")}}
#### CMake Integration

Edit the `CMakeLists.txt` to point to your project's source files:

```cmake
# Update these paths to match your project structure
set(SRC_DIR "../src")               # Path to your source files
include_directories(${SRC_DIR} ../include)  # Include directories
```

Add any additional source files your fuzz target needs:

```cmake
# Add your project's source files
set(PROJECT_SOURCES
    ${SRC_DIR}/module1.c
    ${SRC_DIR}/module2.c
)
set(FUZZ_SRC "${SRC_DIR}/{{target_name}}.c" ${PROJECT_SOURCES})
```
{{/if}}

### 3. Update Test Cases

Replace the example test cases in `testcases/` with inputs relevant to your 
project:

```bash
# Remove example files
rm testcases/example.txt

# Add your own test cases
echo "your test input" > testcases/input1.txt
echo "another test" > testcases/input2.txt
```

### 4. Update Dictionary

Edit `dictionaries/{{target_name}}.dict` with keywords, constants, and strings 
relevant to your project:

```
"MAGIC_VALUE"
"HEADER_SIGNATURE"
"important_keyword"
"\x00\x01\x02\x03"
```

## Building and Running

{{#if (eq integration "makefile")}}
### Using Makefile

```bash
cd fuzz/

# Build different fuzzer variants
make standalone     # No fuzzer engine required
make afl           # AFL fuzzer
make libfuzzer     # libFuzzer
make honggfuzz     # HonggFuzz

# Run fuzzers
make run-standalone
make run-afl
make run-libfuzzer
make run-honggfuzz

# Run smoke test
make test

# Clean build artifacts
make clean
```
{{/if}}

{{#if (eq integration "cmake")}}
### Using CMake

```bash
cd fuzz/
mkdir build && cd build

# Configure
cmake ..

# Build different fuzzer variants
cmake --build . --target {{target_name}}-standalone
cmake --build . --target {{target_name}}-afl
cmake --build . --target {{target_name}}-libfuzzer
cmake --build . --target {{target_name}}-honggfuzz

# Run fuzzers
cmake --build . --target run-standalone
cmake --build . --target run-afl
cmake --build . --target run-libfuzzer
cmake --build . --target run-honggfuzz

# Run smoke test
cmake --build . --target test
```
{{/if}}

### Using Standalone Build Script

If you prefer not to integrate with your build system:

```bash
cd fuzz/
./build.sh                 # Build standalone
USE_AFL=1 ./build.sh       # Build with AFL
USE_LIBFUZZER=1 ./build.sh # Build with libFuzzer
```

## Integration with Your Main Build System

{{#if (eq integration "makefile")}}
### Makefile Integration

You can integrate fuzzing into your main Makefile by including the fuzz 
Makefile:

```makefile
# In your main Makefile
.PHONY: fuzz-test fuzz-clean

fuzz-test:
	$(MAKE) -C fuzz test

fuzz-clean:
	$(MAKE) -C fuzz clean

# Add to your main clean target
clean: fuzz-clean
	# your existing clean commands
```
{{/if}}

{{#if (eq integration "cmake")}}
### CMake Integration

You can integrate fuzzing into your main CMakeLists.txt:

```cmake
# In your main CMakeLists.txt
option(BUILD_FUZZING "Build fuzzing targets" OFF)

if(BUILD_FUZZING)
    add_subdirectory(fuzz)
endif()
```

Then enable fuzzing with:
```bash
cmake -DBUILD_FUZZING=ON ..
```
{{/if}}

## Continuous Integration

Add fuzzing to your CI pipeline:

{{#if (eq integration "makefile")}}
```yaml
# GitHub Actions example
- name: Build and test fuzzer
  run: |
    cd fuzz
    make test
```
{{/if}}

{{#if (eq integration "cmake")}}
```yaml
# GitHub Actions example
- name: Build and test fuzzer
  run: |
    cd fuzz
    mkdir build && cd build
    cmake ..
    cmake --build . --target test
```
{{/if}}

## Dependencies

The fuzzing setup requires:

- **clang** (recommended compiler)
- **AFL++** (for AFL fuzzing)
- **HonggFuzz** (for HonggFuzz)
- **libFuzzer** (usually included with clang)

Install dependencies:

{{#if (eq integration "makefile")}}
```bash
make install-deps
```
{{/if}}

{{#if (eq integration "cmake")}}
```bash
cmake --build . --target install-deps
```
{{/if}}

Or manually:
```bash
# Ubuntu/Debian
sudo apt-get install afl++ honggfuzz

# macOS
brew install afl-fuzz honggfuzz
```

## Troubleshooting

### Common Issues

1. **Compilation errors**: Check that your source paths in the build files are 
   correct and that all necessary dependencies are included.

2. **Fuzzer not found**: Install the required fuzzer engines or use the 
   standalone version.

3. **Permission errors**: Make sure build scripts are executable:
   ```bash
   chmod +x build.sh
   ```

### Getting Help

- Check the README.md for general fuzzing information
- Review your project's source paths in build files
- Ensure all project dependencies are available during fuzzing builds

## Next Steps

1. Implement your fuzz target in `src/{{target_name}}.c`
2. Update source paths in build files
3. Add relevant test cases and dictionary entries
4. Run smoke test to verify setup
5. Integrate with your CI/CD pipeline
6. Start fuzzing!