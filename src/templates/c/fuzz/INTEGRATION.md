# Integrating Fuzzing with Your Build System

This directory contains a flexible fuzzing scaffold that can be adapted to your existing project and build system.

## Integration Overview

The fuzzing setup is designed to work alongside your existing project without modifying your main build system. All fuzzing-specific files are contained in this `fuzz/` directory.

**Key Design Principle**: The fuzz harness needs access to your code, but HOW it gets that access depends on your project structure. This guide shows you multiple approaches to choose from.

## Choose Your Integration Approach

Select the approach that best matches your project structure:

### Approach 1: Library Linking (Recommended)
**Best for**: Projects that already build static libraries or can easily be modified to do so.

**How it works**: Your project builds a static library (e.g., `libmyproject.a`), and the fuzz harness links against it using standard `-L` and `-l` flags.

**Pros**: Clean separation, no recompilation of project code, standard linking
**Cons**: Requires library build capability

### Approach 2: Direct Source Compilation  
**Best for**: Projects with source files that can be compiled directly, monolithic codebases.

**How it works**: The fuzz build process includes your source files directly in the compilation.

**Pros**: Works with any project structure, simple setup
**Cons**: Recompiles project code with fuzzing flags, may expose more internals

### Approach 3: Object File Linking
**Best for**: Projects with complex build systems that produce object files you can reuse.

**How it works**: Link against pre-built object files from your main build.

**Pros**: Reuses existing build artifacts, good for complex projects
**Cons**: Must ensure compatible compilation flags

### Approach 4: Hybrid
**Best for**: Complex projects where different parts need different approaches.

**How it works**: Combine library linking, source compilation, and object files as needed.

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

### Step 1: Implement Your Fuzz Target

Edit `src/{{target_name}}.c` and implement your fuzzing logic in the `LLVMFuzzerTestOneInput` function:

```c
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
// Include your project headers here
#include "your_project.h"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Handle empty input
    if (size == 0) return 0;
    
    // Null-terminate string inputs if needed
    char* input = malloc(size + 1);
    if (!input) return 0;
    memcpy(input, data, size);
    input[size] = '\0';
    
    // Call your project functions
    your_parse_function(input);
    
    free(input);
    return 0;
}
```

### Step 2: Choose and Implement Your Integration Approach

#### Approach 1: Library Linking (Recommended)

If your project can build a static library, this is the cleanest approach:

```bash
# 1. First, make your project build a library
# In your main project directory:
make libmyproject.a     # Or however your project builds libraries

# 2. Edit the fuzz/Makefile to link against it:
```

Edit the relevant sections in your build file:

**For Makefile integration:**
```makefile
# In fuzz/Makefile, update these lines:
INCLUDES = -I../../include -I../../src  # Path to your headers
LIBPATH = -L../..                       # Path to your library
LIBS = -lmyproject                       # Your library name (without lib prefix)

# The build targets will automatically use these
```

**For CMake integration:**
```cmake
# In fuzz/CMakeLists.txt:
target_include_directories(target_name PRIVATE ../../include)
target_link_libraries(target_name PRIVATE ../../libmyproject.a)
```

**For standalone build.sh:**
```bash
# Edit build.sh to add library linking
LDFLAGS="-L../.. -lmyproject"
```

#### Approach 2: Direct Source Compilation

If you can't or don't want to build a library, include your sources directly:

**For Makefile integration:**
```makefile
# In fuzz/Makefile, add your sources:
PROJECT_SOURCES = ../../src/parser.c \
                  ../../src/validator.c \
                  ../../src/utils.c

INCLUDES = -I../../include -I../../src

# Update build targets to include PROJECT_SOURCES:
$(STANDALONE_BIN): $(FUZZ_SRC) $(DRIVER_SRC) $(PROJECT_SOURCES)
	$(CC) $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_STANDALONE \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_SOURCES) -o $@

$(LIBFUZZER_BIN): $(FUZZ_SRC) $(DRIVER_SRC) $(PROJECT_SOURCES)
	clang $(CFLAGS) $(INCLUDES) -fsanitize=fuzzer,address \
		-DFUZZER_TYPE_LIBFUZZER \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_SOURCES) -o $@

# Repeat for other fuzzer targets...
```

**For CMake integration:**
```cmake
# In fuzz/CMakeLists.txt:
set(PROJECT_SOURCES
    ../../src/parser.c
    ../../src/validator.c
    ../../src/utils.c
)

target_sources(target_name PRIVATE ${PROJECT_SOURCES})
target_include_directories(target_name PRIVATE ../../include ../../src)
```

#### Approach 3: Object File Linking

If your project builds object files you can reuse:

**For Makefile integration:**
```makefile
# In fuzz/Makefile:
PROJECT_OBJS = ../../build/parser.o \
               ../../build/validator.o \
               ../../build/utils.o

INCLUDES = -I../../include

$(STANDALONE_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	$(CC) $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_STANDALONE \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_OBJS) -o $@
```

**For CMake integration:**
```cmake
# In fuzz/CMakeLists.txt:
target_link_libraries(target_name PRIVATE 
    ../../build/parser.o
    ../../build/validator.o
    ../../build/utils.o
)
```

#### Approach 4: Hybrid Approach

For complex projects, combine approaches as needed:

```makefile
# Link against some libraries, include some sources, use some objects
LIBS = -L../../lib -lcore
PROJECT_SOURCES = ../../src/special_module.c
PROJECT_OBJS = ../../build/legacy.o

$(LIBFUZZER_BIN): $(FUZZ_SRC) $(DRIVER_SRC) $(PROJECT_SOURCES)
	clang $(CFLAGS) $(INCLUDES) -fsanitize=fuzzer,address \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBS) -o $@
```

### Step 3: Update Test Cases and Dictionary

Replace the example test cases with inputs relevant to your project:

```bash
cd testcases/
rm -f *  # Remove example files

# Add your own test cases
echo "your test input" > input1.txt
echo "another test case" > input2.txt
echo '{"key": "value"}' > json_input.json  # For JSON parsers
printf "\x89PNG\r\n\x1a\n" > png_header.bin  # For binary formats
```

Edit `dictionaries/{{target_name}}.dict` with domain-specific keywords:

```
# For a JSON parser:
"{"
"}"
"null"
"true"
"false"

# For a network protocol:
"GET"
"POST"
"HTTP/1.1"
"Content-Length"

# For your specific domain:
"MAGIC_CONSTANT"
"SIGNATURE_BYTES"
"\x00\x01\x02\x03"
```

## Building and Running

{{#if (eq integration "make")}}
### Using Makefile Integration (Your Configuration)

Since you selected **Makefile integration** with **{{default_fuzzer}}** as your default fuzzer:

```bash
cd fuzz/

# Build your default fuzzer
make {{default_fuzzer}}

# Run quick smoke test
make test

# Run your fuzzer
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

### Building Other Fuzzer Types

```bash
# Build additional fuzzer types (if fuzzer engines are installed)
{{#if (eq default_fuzzer "standalone")}}
make afl            # Requires AFL++
make libfuzzer      # Requires Clang with libFuzzer
make honggfuzz      # Requires HonggFuzz
{{/if}}
{{#if (eq default_fuzzer "afl")}}
make standalone     # No fuzzer engine required
make libfuzzer      # Requires Clang with libFuzzer
make honggfuzz      # Requires HonggFuzz
{{/if}}
{{#if (eq default_fuzzer "libfuzzer")}}
make standalone     # No fuzzer engine required
make afl            # Requires AFL++
make honggfuzz      # Requires HonggFuzz
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
make standalone     # No fuzzer engine required
make afl            # Requires AFL++
make libfuzzer      # Requires Clang with libFuzzer
{{/if}}
```
{{/if}}

{{#if (eq integration "cmake")}}
### Using CMake Integration (Your Configuration)

Since you selected **CMake integration** with **{{default_fuzzer}}** as your default fuzzer:

```bash
cd fuzz/
mkdir build && cd build

cmake ..
cmake --build . --target {{target_name}}-{{default_fuzzer}}
cmake --build . --target test

# Run your fuzzer
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

### Building Other Fuzzer Types

```bash
# Build additional fuzzer types
cmake --build . --target {{target_name}}-standalone
cmake --build . --target {{target_name}}-afl
cmake --build . --target {{target_name}}-libfuzzer
cmake --build . --target {{target_name}}-honggfuzz
```
{{/if}}

{{#if (eq integration "standalone")}}
### Using Standalone Build Script (Your Configuration)

Since you selected **standalone integration** with **{{default_fuzzer}}** as your default fuzzer:

```bash
cd fuzz/

# Build with your default fuzzer
./build.sh          # Builds {{default_fuzzer}} by default
./{{target_name}}-{{default_fuzzer}} testcases/

# Run smoke test
echo "Test input" | ./{{target_name}}-{{default_fuzzer}}
```

### Building Other Fuzzer Types

```bash
# Build with specific fuzzer types
USE_STANDALONE=1 ./build.sh
USE_AFL=1 ./build.sh
USE_LIBFUZZER=1 ./build.sh
USE_HONGGFUZZ=1 ./build.sh
```
{{/if}}

## Integration with Your Main Build System

Add fuzzing targets to your main build system for convenience:

### Main Makefile Integration

```makefile
# In your project's main Makefile:
.PHONY: fuzz-build fuzz-test fuzz-clean

fuzz-build:
	$(MAKE) -C fuzz

fuzz-test:
	$(MAKE) -C fuzz test

fuzz-clean:
	$(MAKE) -C fuzz clean

# Add to your existing clean target
clean: fuzz-clean
	# your existing clean commands
```

### Main CMake Integration

```cmake
# In your project's main CMakeLists.txt:
option(BUILD_FUZZING "Build fuzzing targets" OFF)

if(BUILD_FUZZING)
    add_subdirectory(fuzz)
endif()
```

Enable with: `cmake -DBUILD_FUZZING=ON ..`

## Continuous Integration

Add fuzzing to your CI pipeline:

```yaml
# GitHub Actions example
- name: Build and test fuzzer
  run: |
    cd fuzz
    make test  # or appropriate build command for your setup
```

## Troubleshooting

### Common Issues

1. **Library not found**: 
   - Check that your library exists in the expected location
   - Verify library name matches what's specified in LIBS
   - For approach 1, ensure the library is built before running fuzzer builds

2. **Header files not found**:
   - Check INCLUDES paths point to correct directories
   - Ensure all necessary headers are accessible
   - Verify relative paths are correct from fuzz/ directory

3. **Compilation errors**:
   - Check that source file paths are correct
   - Ensure compatible compiler flags between main project and fuzzer
   - For approach 3, verify object files were built with compatible flags

4. **Linking errors**:
   - For library approach: check library contains expected symbols
   - For direct sources: ensure all required source files are included
   - Check for missing dependencies (system libraries, etc.)

### Getting Help

- Check the README.md for general fuzzing information
- Verify your integration approach matches your project structure
- Test your build modifications incrementally
- Use `make clean && make standalone` to test basic building first

## Next Steps

1. ✅ Choose integration approach based on your project structure
2. ✅ Edit the build files (Makefile/CMakeLists.txt/build.sh) with your paths
3. ✅ Implement your fuzz target in `src/{{target_name}}.c`
4. ✅ Add relevant test cases and dictionary entries
5. ✅ Run smoke test: `make test` or equivalent
6. ✅ Start fuzzing: `make libfuzzer && ./{{target_name}}-libfuzzer testcases/`
7. ✅ Integrate with your CI/CD pipeline
8. ✅ Scale up for longer fuzzing campaigns