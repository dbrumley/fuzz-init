# {{project_name}} Fuzzing Integration Guide

## 🎯 Step 1: Identify Your Fuzz Targets

{{#if suggested_targets}}
Based on your {{project_name}} project analysis, here are **high-value** fuzzing targets:

### **Recommended Primary Targets:**
{{#each suggested_targets}}
- **`{{this.function}}`** - {{this.description}}{{#if this.primary}} ⭐ **START HERE**{{/if}}
{{/each}}

{{else}}
**Find high-value targets in your {{project_name}} project:**

Look for functions that:
- Parse external input (`parse*`, `decode*`, `read*`)
- Process untrusted data (file formats, network input, user input)
- Take string, stream, or buffer parameters
- Have complex logic or memory management

**Quick discovery:**
```bash
# Search your headers for parsing functions:
grep -r "parse.*(" include/ 
grep -r "decode.*(" include/
grep -r "read.*(" include/
```

{{/if}}

---

## 🚀 Step 2: Implement Your First Target

**Replace the demo code** in `src/{{target_name}}.cpp` with a real target from your {{project_name}} library:

### Template for String/Text Processing

```cpp
#include <cstdint>
#include <cstddef>
#include <string>
#include <memory>

// TODO: Add your project headers here
// #include "your_project/parser.h"
// #include "your_project/decoder.h"

extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
    if (size == 0) return 0;
    
    // Convert fuzz input to string
    std::string input(reinterpret_cast<const char*>(data), size);
    
    try {
        // TODO: Replace with your target function
        // Examples:
        // auto result = your_project::parse_document(input);
        // auto result = your_project::decode_message(input);
        // your_project::process_config(input);
        
        // Force evaluation if needed
        // if (result) { (void)result->some_method(); }
        
    } catch (const std::exception& e) {
        // Expected for malformed input - don't crash the fuzzer
        return 0;
    } catch (...) {
        // Catch any other exceptions
        return 0;
    }
    
    return 0;
}
```

### Template for Binary Data Processing

```cpp
extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
    if (size == 0) return 0;
    
    try {
        // TODO: Replace with your binary processing function
        // Examples:
        // your_project::parse_binary_format(data, size);
        // your_project::decode_packet(data, size);
        // your_project::process_image_data(data, size);
        
    } catch (...) {
        // Handle exceptions gracefully
    }
    
    return 0;
}
```

{{#if suggested_targets}}
### Concrete Example for {{project_name}}

```cpp
{{suggested_example_code}}
```
{{/if}}

---

## ⚡ Step 3: Build and Test

{{#if (eq integration "cmake")}}
### Quick Setup (Auto-Configuration)
```bash
cd fuzz
./configure.sh  # Handles CMake setup and building
```

### Manual Setup (If configure.sh fails)
```bash
cd fuzz
mkdir build && cd build
cmake -S .. -B .
cmake --build . --target {{target_name}}_{{default_fuzzer}}
```

### Test Your Target
```bash
# Quick smoke test
echo 'test input' | ./{{target_name}}_{{default_fuzzer}}

# Real fuzzing session
./{{target_name}}_{{default_fuzzer}} ../testsuite/ -dict=../dictionaries/{{target_name}}.dict -max_total_time=300
```

{{else}}
### Build Your Fuzzer
```bash
cd fuzz
make {{default_fuzzer}}  # or: make all
```

### Test Your Target
```bash
# Quick smoke test
echo 'test input' | ./{{target_name}}-{{default_fuzzer}}

# Real fuzzing session
./{{target_name}}-{{default_fuzzer}} testsuite/
```
{{/if}}

---

## 🔧 Step 4: Optimize for Better Bug Finding

### Enable Full Sanitizers (Critical!)

{{#if (eq integration "cmake")}}
For effective bug finding, rebuild {{project_name}} with AddressSanitizer:

```bash
# From {{project_name}} project root
cmake -S . -B build-asan -DCMAKE_CXX_FLAGS="-fsanitize=address -g -O1"
cmake --build build-asan

# Rebuild fuzzer with sanitizer integration
cd fuzz
rm -rf build
cmake -S . -B build -DPARENT_BUILD_DIR=../build-asan
cmake --build build --target {{target_name}}_{{default_fuzzer}}
```

{{else}}
For effective bug finding, rebuild {{project_name}} with AddressSanitizer:

```bash
# Rebuild your main project with sanitizers
make clean
CXXFLAGS="-fsanitize=address -g" make

# Rebuild fuzzer
cd fuzz
make clean && make {{default_fuzzer}}
```
{{/if}}

### Improve Test Cases

Replace generic test cases with {{project_name}}-specific ones:

```bash
cd testsuite/{{target_name}}/
rm demo_crash.txt safe.txt

# TODO: Add real test cases for your project
# Examples:
# echo "valid input" > valid_input.txt
# echo "edge case" > edge_case.txt
# printf "\x00\x01\x02\x03" > binary_input.bin
```

### Enhance Dictionary

Edit `dictionaries/{{target_name}}.dict` with {{project_name}}-specific keywords:

```
# TODO: Add keywords specific to your input format
# Examples:
# "magic_header"
# "version"
# "null"
# "true"
# "false"
# "\x00\x01"
```

---

## 📈 Step 5: Add More Targets (Advanced)

### Multiple Harnesses

Create additional harnesses for different targets:

```bash
# Copy base harness
cp src/{{target_name}}.cpp src/second_target.cpp

# Edit src/second_target.cpp to fuzz a different function
```

{{#if (eq integration "cmake")}}
### Update CMakeLists.txt for Multiple Targets

```cmake
# Add after the main target definition in CMakeLists.txt:

# Additional fuzzing targets (uncomment and modify):
# add_executable(second_target_{{default_fuzzer}} src/second_target.cpp)
# target_compile_options(second_target_{{default_fuzzer}} PRIVATE ${COMMON_FLAGS} ${FUZZER_FLAGS})
# target_link_libraries(second_target_{{default_fuzzer}} PRIVATE ${REQUIRED_LIBRARY_TARGET})
# target_link_options(second_target_{{default_fuzzer}} PRIVATE ${FUZZER_FLAGS})
```

### Build Additional Targets

```bash
cmake --build build --target second_target_{{default_fuzzer}}
./second_target_{{default_fuzzer}} ../testsuite/
```

{{else}}
### Update Makefile for Multiple Targets

Add new targets to your `Makefile`:

```makefile
# Add new fuzzer targets
second_target-{{default_fuzzer}}: src/second_target.cpp $(DRIVER_SRC)
	$(CXX) $(CXXFLAGS) $(INCLUDES) $(FUZZER_FLAGS) \
		$(DRIVER_SRC) $< $(LIBS) -o $@
```
{{/if}}

---

## ✅ Success Checklist

- ✅ **Identified target function** in {{project_name}} to fuzz
- ✅ **Replaced demo code** with real function calls
- ✅ **Built successfully** with {{#if (eq integration "cmake")}}./configure.sh{{else}}make {{default_fuzzer}}{{/if}}
- ✅ **Smoke test passes** - fuzzer runs without immediate crash
- ✅ **Sanitizers enabled** - rebuilt {{project_name}} with AddressSanitizer
- ✅ **Real test cases** - added {{project_name}}-specific samples
- ✅ **Dictionary updated** - added domain-specific keywords
- ✅ **Finding bugs** - fuzzer discovers crashes in your code

---

## 🔧 Troubleshooting

{{#if (eq integration "cmake")}}
### CMake Issues

**Library not found:**
```bash
# Check that {{project_name}} builds correctly first
cd .. && cmake -S . -B build && cmake --build build
cd fuzz && ./configure.sh
```

{{#if (eq default_fuzzer "libfuzzer")}}
**Wrong compiler:**
```bash
# Ensure clang++ is available for libFuzzer
which clang++
export CXX=clang++
```
{{/if}}

{{else}}
### Build Issues

**Library not found:**
```bash
# Check library paths in Makefile
# Update LIBPATH and LIBS variables as needed
```

**Header files not found:**
```bash
# Check include paths in Makefile
# Update INCLUDES variable with correct paths
```
{{/if}}

### Fuzzing Issues

**No crashes found:**
- Verify sanitizers are enabled (see Step 4)
- Try different targets or input types
- Add more malformed test cases
- Check that exceptions are being caught properly

**Fuzzer crashes immediately:**
- Check that your harness handles exceptions properly
- Test with simple input first: `echo "test" | ./fuzzer`
- Verify library linking is correct

---

## 📚 Detailed Reference ({{integration}} Integration)

### Understanding Your Setup

This fuzzing setup uses **{{integration}} integration** with **{{default_fuzzer}}** as the default fuzzer.

**Generated Files:**
{{#if (eq integration "cmake")}}
- `CMakeLists.txt` - CMake configuration for fuzzing
{{else}}
- `Makefile` - Build configuration for fuzzing
{{/if}}
- `src/{{target_name}}.cpp` - Your fuzz harness (customize this!)
{{#if (eq integration "cmake")}}
- `configure.sh` - Auto-setup script
{{else}}
- `build.sh` - Build script for standalone mode
{{/if}}
- `dictionaries/{{target_name}}.dict` - Fuzzing dictionary
- `testsuite/` - Initial test inputs

{{#if (eq integration "cmake")}}
### CMake Integration Details

**Library Linking Strategy:**

The CMake setup intelligently links against your {{project_name}} library:

1. **Preferred**: Links against fuzzer-specific library (e.g., `{{project_name}}-{{default_fuzzer}}`)
2. **Fallback**: Links against main library target (`{{project_name}}` or `lib{{project_name}}`)
3. **Auto-detection**: Scans for available library targets

**Sanitizer Coordination:**

For effective fuzzing, **both your library AND the fuzzer** need the same sanitizers:

```bash
# Build {{project_name}} with sanitizers
cmake -S . -B build-asan -DCMAKE_CXX_FLAGS="-fsanitize=address -g"
cmake --build build-asan

# Rebuild fuzzer to use sanitizer-instrumented library
cd fuzz && rm -rf build
cmake -S . -B build -DPARENT_BUILD_DIR=../build-asan
cmake --build build
```

### Adding CMake Integration to Main Project

Add fuzzing support to your main CMakeLists.txt:

```cmake
# In main CMakeLists.txt
option(BUILD_FUZZING "Build fuzzing targets" OFF)

if(BUILD_FUZZING)
    # Add sanitizer flags when fuzzing
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address -g -O1")
    
    # Optional: Create fuzzer-specific library target
    add_library({{project_name}}-{{default_fuzzer}} STATIC ${LIB_SOURCES})
    target_compile_options({{project_name}}-{{default_fuzzer}} PRIVATE -fsanitize=address -g)
    target_include_directories({{project_name}}-{{default_fuzzer}} PUBLIC include)
    
    # Enable fuzzing subdirectory
    add_subdirectory(fuzz)
endif()
```

Enable with: `cmake -DBUILD_FUZZING=ON ..`

{{else}}
### Makefile Integration Details

**Library Linking Strategy:**

The Makefile setup provides multiple approaches:

1. **Library Linking**: Link against pre-built libraries
2. **Source Compilation**: Include source files directly
3. **Object Linking**: Link against object files

**Sanitizer Coordination:**

Ensure consistent sanitizer flags between main project and fuzzer:

```makefile
# In main project Makefile
FUZZ_CXXFLAGS = -fsanitize=address -g -O1

# Build library with fuzzing flags
lib{{project_name}}-fuzz.a: $(SOURCES)
	$(CXX) $(CXXFLAGS) $(FUZZ_CXXFLAGS) -c $(SOURCES)
	ar rcs $@ *.o
```

### Integration Approaches

**Approach 1: Library Linking (Recommended)**
```makefile
# In fuzz/Makefile
LIBS = -L.. -l{{project_name}}
INCLUDES = -I../include
```

**Approach 2: Direct Source Compilation**
```makefile
# In fuzz/Makefile
PROJECT_SOURCES = ../src/parser.cpp ../src/utils.cpp
INCLUDES = -I../include -I../src
```

**Approach 3: Object File Linking**
```makefile
# In fuzz/Makefile
PROJECT_OBJS = ../build/parser.o ../build/utils.o
INCLUDES = -I../include
```
{{/if}}

### Performance Optimization

**Faster builds:**
{{#if (eq integration "cmake")}}
```bash
# Use ninja for faster builds
cmake -S . -B build -GNinja
ninja -C build
```
{{else}}
```bash
# Use parallel make
make -j$(nproc)
```
{{/if}}

**Parallel fuzzing:**
```bash
# Run multiple fuzzer instances
./{{target_name}}_{{default_fuzzer}} corpus1/ &
./{{target_name}}_{{default_fuzzer}} corpus2/ &
./{{target_name}}_{{default_fuzzer}} corpus3/ &
wait
```

### Integration with CI/CD

Add to your GitHub Actions:

```yaml
- name: Build and test fuzzing
  run: |
{{#if (eq integration "cmake")}}
    cmake -S . -B build -DBUILD_FUZZING=ON
    cmake --build build
    cd fuzz/build && ./{{target_name}}_{{default_fuzzer}} ../testsuite/ -runs=1000
{{else}}
    cd fuzz && make {{default_fuzzer}}
    ./{{target_name}}-{{default_fuzzer}} testsuite/ -runs=1000
{{/if}}
```

### Common Integration Patterns

**Custom include paths:**
{{#if (eq integration "cmake")}}
```cmake
target_include_directories(${TARGET_NAME} PRIVATE 
    ../external/boost
    ../third_party/includes
)
```
{{else}}
```makefile
INCLUDES = -I../include -I../external/boost -I../third_party/includes
```
{{/if}}

**Link additional libraries:**
{{#if (eq integration "cmake")}}
```cmake
target_link_libraries(${TARGET_NAME} PRIVATE 
    ${REQUIRED_LIBRARY_TARGET}
    Boost::system
    ${CMAKE_DL_LIBS}
)
```
{{else}}
```makefile
LIBS = -L.. -l{{project_name}} -lboost_system -ldl
```
{{/if}}

**Conditional compilation:**
{{#if (eq integration "cmake")}}
```cmake
target_compile_definitions(${TARGET_NAME} PRIVATE
    FUZZING_BUILD
    $<$<CONFIG:Debug>:DEBUG_LOGGING>
)
```
{{else}}
```makefile
CXXFLAGS += -DFUZZING_BUILD -DDEBUG_LOGGING
```
{{/if}}

---

That's it! You now have a complete {{project_name}} fuzzing setup optimized for {{integration}} with {{default_fuzzer}}. Start by implementing your first target, find some bugs, then expand to additional targets. Happy fuzzing! 🐛🎯