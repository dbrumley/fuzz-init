# Integrating {{project_name}} Fuzzing into Your Project

This guide explains how to integrate this fuzzing harness into your existing
C++ project.


{{#if minimal}}
## Drop-in Integration

You've generated a drop-in fuzzing setup that expects to link against your
existing project. You will need to ensure that your **entire project** is built
with the appropriate sanitizers.  

### Prerequisites

We've dropped in a setup that builds: 
- **Clang/libfuzzer** with AddressSanitizer: `-fsanitize=address,undefined`
- **AFL++ compiler**: `afl-clang-fast++`
- **HonggFuzz compiler**: `hfuzz-clang++`
- **Standard (native) compiler** for standalone fuzzing without clang or AFL++.

You can pick and choose which ones you want to use. Generally users pick AFL++
for applications, and libfuzzer for libraries.  If you cannot add
instrumentation, never fear -- just build a native, uninstrumented target and
run in Mayhem. 

### Integration Steps

1. **Place this fuzz directory in your project**:
   ```bash
   cp -r fuzz/ /path/to/your/project/
   ```

2. **Update the harness** (`src/{{target_name}}.cpp`):
   - Replace the demo code with calls to your actual functions
   - Include your project headers
   - Handle exceptions appropriately

3. **Build your library with each fuzzer's compiler**:
   
   {{#if (eq integration 'cmake')}}
   ```bash
   # For libFuzzer
   CC=clang CXX=clang++ cmake -B build-libfuzzer -DCMAKE_CXX_FLAGS="-fsanitize=address,undefined -g -O1"
   cmake --build build-libfuzzer
   
   # For AFL++
   CC=afl-clang-fast CXX=afl-clang-fast++ cmake -B build-afl -DCMAKE_CXX_FLAGS="-fsanitize=address,undefined -g -O1"
   cmake --build build-afl
   
   # Similar for other fuzzers...
   ```
   {{else}}
   ```bash
   # For libFuzzer
   make clean
   CXX=clang++ CXXFLAGS="-fsanitize=address,undefined -g -O1" make
   
   # For AFL++
   make clean
   CXX=afl-clang-fast++ CXXFLAGS="-fsanitize=address,undefined -g -O1" make
   
   # Similar for other fuzzers...
   ```
   {{/if}}

4. **Build the fuzz harnesses**:
   {{#if (eq integration 'cmake')}}
   ```bash
   cd fuzz
   cmake --preset fuzz-libfuzzer && cmake --build --preset fuzz-libfuzzer
   cmake --preset fuzz-afl && cmake --build --preset fuzz-afl
   # Or use: ./fuzz.sh build
   ```
   {{else}}
   ```bash
   cd fuzz
   make  # Builds all fuzzers
   # Or: ./fuzz.sh build
   ```
   {{/if}}

{{else}}
## Full Mode Integration

You've generated a complete example project with fuzzing. To integrate into your existing project:

### Option 1: Copy Just the Fuzz Directory

1. **Copy the fuzz directory**:
   ```bash
   cp -r fuzz/ /path/to/your/project/
   ```

2. **Update the build configuration** to point to your library:
   {{#if (eq integration 'cmake')}}
   - Edit `fuzz/CMakeLists.txt`
   - Change library linking from `mylib` to your actual library name
   {{else}}
   - Edit `fuzz/Makefile`
   - Update `LIBPART` to point to your library
   {{/if}}

3. **Update the harness** as described below

### Option 2: Study and Adapt

Review the example implementation to understand:
- How sanitizers are configured in the main `{{#if (eq integration 'cmake')}}CMakeLists.txt{{else}}Makefile{{/if}}`
- How the library is built with fuzzing instrumentation
- How multiple fuzzer engines are supported

{{/if}}

## Updating the Fuzz Harness

Replace the demo code in `src/{{target_name}}.cpp` with your actual target:

```cpp
#include <cstdint>
#include <cstddef>
#include <cstring>

// Include YOUR headers
#include "your_project/parser.h"  // Example

extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
    if (size == 0) return 0;
    
    // Example: Parse untrusted input
    try {
        YourProject::Parser parser;
        parser.parse(data, size);
    } catch (...) {
        // Exceptions are expected for malformed input
        return 0;
    }
    
    return 0;
}
```

## Running the Fuzzers

### Quick Test
```bash
# Test that your harness works
echo "test" | ./fuzz/build/{{target_name}}-libfuzzer

# Run for 60 seconds
./fuzz/build/{{target_name}}-libfuzzer -max_total_time=60
```

### Full Fuzzing Session
```bash
cd fuzz

# Using the unified script (recommended)
./fuzz.sh test              # Quick 10-second test of all engines
./fuzz.sh test libfuzzer 300  # 5-minute libFuzzer run

# Or directly
./build/{{target_name}}-libfuzzer corpus/ -max_total_time=3600
```

### Using Different Engines
- **libFuzzer**: `./build/{{target_name}}-libfuzzer corpus/`
- **AFL++**: `afl-fuzz -i seeds -o afl-out -- ./build/{{target_name}}-afl @@`
- **HonggFuzz**: `hongfuzz -i seeds -o hfuzz-out -- ./build/{{target_name}}-hongfuzz ___FILE___`
- **Standalone**: `./build/{{target_name}}-standalone < testcase.txt`

## Critical Requirements

### 1. Consistent Sanitizer Usage

**The library and fuzzer MUST use the same sanitizers**, otherwise you'll get linker errors or miss bugs:

{{#if minimal}}
- When building for libFuzzer/AFL/HonggFuzz: Use `-fsanitize=address,undefined`
- When building for standalone: No sanitizers needed
- Each fuzzer requires rebuilding your library with its specific compiler
{{else}}
- The example shows how each fuzzer gets its own library build
- Study the root `{{#if (eq integration 'cmake')}}CMakeLists.txt{{else}}Makefile{{/if}}` for the pattern
{{/if}}

### 2. Handling Multiple Targets

To fuzz different functions, create multiple harnesses:

```bash
cp src/{{target_name}}.cpp src/parser_fuzz.cpp
cp src/{{target_name}}.cpp src/decoder_fuzz.cpp
# Edit each to target different functions
```

{{#if (eq integration 'cmake')}}
Update `fuzz/CMakeLists.txt` to build additional targets:
```cmake
set(FUZZ_HARNESS_SRCS 
  "${FUZZ_SRC_DIR}/{{target_name}}.cpp"
  "${FUZZ_SRC_DIR}/parser_fuzz.cpp"
  "${FUZZ_SRC_DIR}/decoder_fuzz.cpp"
)
```
{{else}}
The Makefile will automatically detect new `.cpp` files in `src/`.
{{/if}}

### 3. Seed Corpus

Replace the demo files in `testsuite/{{target_name}}/` with real examples:
```bash
# Add valid inputs your code should handle
cp /path/to/valid/samples/* testsuite/{{target_name}}/

# Add edge cases and previously found bugs
cp /path/to/edge/cases/* testsuite/{{target_name}}/
```

### 4. Dictionary

Update `dictionaries/{{target_name}}.dict` with protocol-specific tokens:
```
# JSON example
"null"
"true" 
"false"
"\"key\":"

# Binary protocol example
"\x00\x00\x00\x01"  # Version 1
"\xff\xff\xff\xff"  # Max value
"MAGIC"             # File header
```

## Troubleshooting

### Linker Errors (undefined reference to `__asan_*`)
- **Cause**: Library built without sanitizers, fuzzer built with sanitizers
- **Fix**: Rebuild library with matching sanitizers

### No Crashes Found
- **Cause**: Missing sanitizers or catching all errors
- **Fix**: Ensure both library and fuzzer use `-fsanitize=address,undefined`
- **Fix**: Let some errors through (array bounds, null derefs, etc.)

### Build Can't Find Library
{{#if (eq integration 'cmake')}}
- Check `PARENT_BUILD_DIR` in CMake configuration
- Ensure library target is built before fuzzing
{{else}}
- Update `LIBPART` in fuzz/Makefile to correct path
- Check library name matches (lib{{project_name}}.a vs {{project_name}}.a)
{{/if}}

## Next Steps

1. ✅ Get a basic harness working with your code
2. ✅ Run for 5-10 minutes to verify it finds bugs
3. ✅ Add more test cases to the corpus
4. ✅ Create additional harnesses for other entry points
5. ✅ Integrate into CI/CD for regression testing

For more details, see:
- `README.md` - Quick reference and commands
- `fuzz.sh` - Unified build/test script
- Example harness in `src/{{target_name}}.cpp`