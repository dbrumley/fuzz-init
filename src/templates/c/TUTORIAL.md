# {{project_name}} Fuzzing Tutorial

Welcome to the complete fuzzing tutorial for {{project_name}}! 

This guide will walk you through the entire process of setting up and running 
fuzzing on your software.

## What is Fuzzing?

Fuzzing is an automated dynamic analysis technique that discovers software bugs
by systematically generating inputs designed to explore new program behaviors.
Rather than relying on purely random data, modern fuzzers like AFL, libFuzzer,
and HonggFuzz use feedback mechanisms — such as code coverage, execution
traces, or sanitizers — to guide input generation toward previously unexplored
execution paths. These tools evolve inputs using strategies like mutation,
minimization, and corpus management to maximize path discovery and uncover edge
cases that lead to crashes, memory errors, and other vulnerabilities.

Commercial tools like Mayhem take it up a notch by adding
sophisticated program analysis, such as symbolic execution, to get even better
results.

## Project Structure

Your fuzzing project has been set up with the following structure:

```
{{project_name}}/
├── src/                            # Example library source code
│   ├── gps.c                       # GPS parser implementation
│   └── main.c                      # Example application
├── include/
│   └── gps.h                       # Public library interface
├── Makefile                        # Builds library and delegates to fuzz
├── libgps.a                        # Generated library (after 'make lib')
├── fuzz/                           # All fuzzing-related files
│   ├── src/{{target_name}}.c       # Fuzz harness that uses the library
│   ├── driver/main.c               # Universal fuzzer driver
│   ├── Makefile                    # Links against parent library
│   ├── testsuite/                  # Initial test inputs
│   ├── dictionaries/               # Fuzzing dictionaries
│   ├── Dockerfile                  # Container for reproducible fuzzing
│   ├── Mayhemfile                  # Mayhem.security configuration
│   ├── INTEGRATION.md              # Guide for integrating with your projects
│   └── README.md                   # Quick reference guide
└── TUTORIAL.md                     # This comprehensive tutorial
```

This example demonstrates the **ideal library-based approach** for fuzzing integration, where:
- Your main code is built into a library (`libgps.a`)
- The fuzz harness links against this library cleanly
- Application logic for reading in input is in a separate file (`main.c`)

## Fuzzing 101

Setting up fuzzing means creating a dynamic analysis environment where
your application:
  - Reads in input from a file or stdin.
  - Calls functions you wish to test.
  - Is instrumented with sanitizers that trigger when a bug is detected.

Setting up your application this way will always result in the best fuzzing
performance, and detecting the most bugs.


Often a single code base will result in multiple **fuzz targets** (aka a
**harness**), which are stand-alone executables that exercise different
functions. 

It's recommended to also include any known tests to the fuzz target. The set of 
tests is called the **fuzz testsuite**, aka a **fuzz corpus**.  The testsuite
bootstraps fuzzing so it doesn't have to re-discover code paths you've already
written tests for.  


**But what if you can't recompile? Or your application reads from tcp or udp
sockets?** Then you must use a commercial solution like
[Mayhem](https://app.mayhem.security). 


## Step 1: Understanding the Fuzz Target

Open `fuzz/src/{{target_name}}.c` and examine the fuzz target:

```c
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <gps.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Null-terminate the input data
    char* input = (char*)malloc(size + 1);
    if (!input) return 0;
    
    memcpy(input, data, size);
    input[size] = '\\0';
    
    // Parse GPS data using the library
    gps_coordinate_t coord;
    int result = parse_nmea_line(input, &coord);
    
    // If parsing succeeded, process the coordinate
    if (result == 0 && coord.valid) {
        // Test all bug triggers (0 = all bugs)
        process_coordinate(coord, 0);
    }
    
    free(input);
    return 0;
}
```

This demonstrates **key fuzzing principles**:

1. **Universal Entry Point**: The `LLVMFuzzerTestOneInput` function is the standard interface that works with all fuzzing engines (AFL, libFuzzer, HonggFuzz, standalone)

2. **Input Processing**: Takes raw bytes from the fuzzer and converts them into the format your code expects (here, null-terminated strings for GPS parsing)

3. **Target Coverage**: Calls library functions that exercise complex logic where bugs might hide

4. **Error Handling**: Gracefully handles invalid inputs without crashing (unless it's a real bug)

5. **Cross-Fuzzer Compatibility**: This same code works with:
   - **AFL/AFL++**: Coverage-guided fuzzing with persistent mode
   - **libFuzzer**: Built-in Clang fuzzing engine with structure-aware mutations  
   - **HonggFuzz**: Alternative feedback-driven fuzzer
   - **Standalone**: Regular executable for debugging and manual testing

## Step 2: How Fuzzing Works

This example demonstrates **coverage-guided fuzzing** - the most effective automated bug-finding technique:

### The Fuzzing Loop

1. **Generate Input**: Fuzzer creates test inputs (starting from seed files in `testsuite/`)
2. **Execute Target**: Runs your code with the input, tracking which code paths are hit
3. **Measure Coverage**: Instruments the binary to see what new code was reached
4. **Mutate & Evolve**: Keeps inputs that found new code paths, mutates them to explore further
5. **Detect Crashes**: When bugs cause crashes, the fuzzer saves the crashing input

### Why This Example Works Well

The GPS parser is an ideal fuzzing target because it:
- **Processes Untrusted Input**: Takes string data that could come from anywhere
- **Has Complex Logic**: Parsing involves many code paths and edge cases  
- **Contains Real Bugs**: Intentional vulnerabilities that fuzzing will discover
- **Fast Execution**: Runs quickly enough for millions of iterations

### The Library-Based Architecture

```bash
# 1. Build the library first  
make lib          # Creates libgps.a from src/gps.c

# 2. Build fuzzers that link against it
make fuzz         # Default fuzzer (delegates to fuzz/Makefile)
```

This demonstrates **fuzzing best practices**:
- **Clean Interface**: Fuzzer links against library, not individual source files
- **Realistic Testing**: Tests the same code your application would use
- **Easy Integration**: Drop fuzzing into existing projects without restructuring
- **Cross-Fuzzer Support**: Same library works with AFL, libFuzzer, HonggFuzz

The fuzzing Makefile shows the standard pattern:

```makefile
# In fuzz/Makefile
LIBPATH = -L..
LIBS = -lgps
INCLUDES = -I../include
```

**Key Insight**: The fuzzer doesn't need to know about your internal source structure - it just links against your library like any other application would.

## Step 3: Adapting to Your Own Projects

The key insight is that you have **multiple integration options** depending on your project structure:

### Option 1: Library-Based (Recommended)
If your project can build a library:
```bash
# Your project builds libmyproject.a
make libmyproject.a

# Edit fuzz/Makefile to link against it
LIBS = -L../../lib -lmyproject
```

### Option 2: Direct Source Compilation
For projects that don't build libraries:
```bash
# Edit fuzz/Makefile to include your sources directly
PROJECT_SOURCES = ../../src/parser.c ../../src/utils.c
```

### Option 3: Mixed Approach
Combine libraries and sources as needed.

**For detailed integration instructions, see `fuzz/INTEGRATION.md`** - it provides step-by-step guidance for adapting this scaffolding to your specific project structure.

## Step 4: Customize Your Fuzz Target

Replace the GPS example with your actual testing logic:

1. **Include your headers**: Add includes for the code you want to test
2. **Process the input**: Parse the fuzzer-provided data appropriately
3. **Call your functions**: Invoke the code you want to fuzz
4. **Handle errors gracefully**: Don't crash on invalid input unless it's a real bug

Example for testing a JSON parser:

```c
#include <stdint.h>
#include <stddef.h>
#include "your_json_parser.h"  // Your project header

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Ensure null termination for string functions
    if (size == 0) return 0;

    char *json_str = malloc(size + 1);
    memcpy(json_str, data, size);
    json_str[size] = '\\0';

    // Test your JSON parser
    json_object *obj = parse_json(json_str);
    if (obj) {
        // Test additional functions on valid JSON
        char *serialized = json_to_string(obj);
        free(serialized);
        json_free(obj);
    }

    free(json_str);
    return 0;
}
```

## Step 5: Build and Test

{{#if (eq integration "make")}}
Your project is configured for **Makefile integration** with **{{default_fuzzer}}** as the default fuzzer.

```bash
# 1. First, build the library (required)
make lib    # Creates libgps.a

# 2. Build and test your selected fuzzer
make fuzz             # Builds {{default_fuzzer}} fuzzer
make fuzz-test        # Quick smoke test

# 3. Run your fuzzer
cd fuzz/
./{{target_name}}-{{default_fuzzer}} testsuite/
{{#if (eq default_fuzzer "libfuzzer")}}
./{{target_name}}-{{default_fuzzer}} -dict=dictionaries/{{target_name}}.dict testsuite/
{{/if}}
{{#if (eq default_fuzzer "afl")}}
mkdir -p findings
afl-fuzz -i testsuite -o findings -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
mkdir -p corpus
honggfuzz -i testcases -W corpus -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
```

### Additional Fuzzer Options

You can also build other fuzzer types:

```bash
{{#if (eq default_fuzzer "standalone")}}
make fuzz-afl         # Requires AFL++ installation
make fuzz-libfuzzer   # Requires Clang with libFuzzer
make fuzz-honggfuzz   # Requires HonggFuzz installation
{{/if}}
{{#if (eq default_fuzzer "afl")}}
make fuzz-standalone  # No fuzzer engine required
make fuzz-libfuzzer   # Requires Clang with libFuzzer
make fuzz-honggfuzz   # Requires HonggFuzz installation
{{/if}}
{{#if (eq default_fuzzer "libfuzzer")}}
make fuzz-standalone  # No fuzzer engine required
make fuzz-afl         # Requires AFL++ installation
make fuzz-honggfuzz   # Requires HonggFuzz installation
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
make fuzz-standalone  # No fuzzer engine required
make fuzz-afl         # Requires AFL++ installation
make fuzz-libfuzzer   # Requires Clang with libFuzzer
{{/if}}
```
{{/if}}

{{#if (eq integration "standalone")}}
Your project is configured for **standalone integration** using build scripts.

```bash
# 1. First, build the library (required)
make lib    # Creates libgps.a

# 2. Build and test the fuzzer
make fuzz             # Uses build.sh to build {{default_fuzzer}}
make fuzz-test        # Quick smoke test

# 3. Run the fuzzer manually
cd fuzz/
./{{target_name}}-{{default_fuzzer}} testsuite/
{{#if (eq default_fuzzer "libfuzzer")}}
./{{target_name}}-{{default_fuzzer}} -dict=dictionaries/{{target_name}}.dict testsuite/
{{/if}}
{{#if (eq default_fuzzer "afl")}}
mkdir -p findings
afl-fuzz -i testsuite -o findings -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
mkdir -p corpus
honggfuzz -i testcases -W corpus -- ./{{target_name}}-{{default_fuzzer}}
{{/if}}
```

### Alternative Build Script Usage

You can also use the build script directly with different fuzzer types:

```bash
cd fuzz/

# Build with your default fuzzer
./build.sh
./{{target_name}}-{{default_fuzzer}} testsuite/

# Build with specific fuzzers
USE_AFL=1 ./build.sh
USE_LIBFUZZER=1 ./build.sh
USE_HONGGFUZZ=1 ./build.sh
USE_STANDALONE=1 ./build.sh
```
{{/if}}

{{#if (eq integration "cmake")}}
Your project is configured for **CMake integration** with **{{default_fuzzer}}** as the default fuzzer.

```bash
# 1. First, build the library (required)
make lib    # Creates libgps.a

# 2. Build and test the fuzzer
cd fuzz/
mkdir build && cd build
cmake ..
cmake --build . --target {{target_name}}-{{default_fuzzer}}

# 3. Run the fuzzer
./{{target_name}}-{{default_fuzzer}} ../testsuite/
{{#if (eq default_fuzzer "libfuzzer")}}
./{{target_name}}-{{default_fuzzer}} -dict=../dictionaries/{{target_name}}.dict ../testsuite/
{{/if}}
```

### Additional Fuzzer Types

```bash
# Build other fuzzer types
cmake --build . --target {{target_name}}-standalone
cmake --build . --target {{target_name}}-afl  
cmake --build . --target {{target_name}}-libfuzzer
cmake --build . --target {{target_name}}-honggfuzz
```
{{/if}}

## Step 4: Customize Test Cases and Dictionaries

### Test Cases

Replace the example test cases in `testsuite/` with inputs relevant to your code:

```bash
cd testsuite/
rm example.txt crash.txt

# Add your own test cases
echo '{"key": "value"}' > valid.json
echo '{"incomplete":' > invalid.json
python -c "print('A' * 1000)" > large_input.txt
```

### Dictionaries

Edit `dictionaries/{{target_name}}.dict` with keywords specific to your domain:

```
# JSON parsing dictionary
"{"
"}"
"["
"]"
"null"
"true"
"false"
"key"
"value"
"\\"
```

## Step 5: Run Extended Fuzzing Campaigns

For serious bug hunting, run longer fuzzing campaigns:

### AFL Campaign

```bash
# Terminal 1: Main fuzzer
afl-fuzz -i testsuite -o findings -M main -- ./{{target_name}}-afl

# Terminal 2: Secondary fuzzer
afl-fuzz -i testsuite -o findings -S secondary -- ./{{target_name}}-afl

# Check status
afl-whatsup findings/
```

### libFuzzer Campaign

```bash
mkdir corpus
./{{target_name}}-libfuzzer corpus/ testsuite/ \\
    -dict=dictionaries/{{target_name}}.dict \\
    -jobs=4 \\
    -workers=4 \\
    -max_total_time=3600
```

### HonggFuzz Campaign

```bash
honggfuzz \\
    -i testsuite \\
    -W corpus \\
    -w dictionaries/{{target_name}}.dict \\
    -t 60 \\
    -n 4 \\
    -- ./{{target_name}}-honggfuzz
```

## Step 6: Using Docker for Reproducible Fuzzing

The included Dockerfile provides a consistent fuzzing environment:

```bash
# Build the container
docker build -t {{project_name}}-fuzz .

# Run fuzzing in container
docker run -v $(pwd)/findings:/findings {{project_name}}-fuzz
```

## Step 7: Integrate with Mayhem.security

Mayhem.security is a continuous fuzzing platform. Your project includes a `Mayhemfile` for easy integration:

```bash
# Upload to Mayhem (requires account)
mayhem run .
```

## Common Issues and Solutions

### Build Errors

- **Missing headers**: Make sure all dependencies are installed
- **Linker errors**: Check that all required libraries are linked
- **Fuzzer not found**: Install the fuzzing engine or use standalone mode

### Fuzzing Issues

- **No crashes found**: Try more diverse test cases or longer campaigns
- **Fuzzer gets stuck**: Add better seed inputs or improve dictionary
- **Performance issues**: Profile your fuzz target for bottlenecks

### Debugging Crashes

```bash
# Reproduce crash with standalone binary
./{{target_name}}-standalone < findings/crashes/id:000000*

# Debug with GDB
gdb ./{{target_name}}-standalone
(gdb) run < findings/crashes/id:000000*
(gdb) bt
```

## Best Practices

1. **Start Simple**: Begin with basic functionality before complex scenarios
2. **Fast Execution**: Optimize your fuzz target for speed
3. **Good Seeds**: Provide diverse, valid test cases as starting points
4. **Comprehensive Dictionaries**: Include all relevant keywords and values
5. **Monitor Progress**: Check fuzzer statistics regularly
6. **Reproduce Issues**: Always verify crashes in debug builds
7. **Fix and Retest**: After fixing bugs, run fuzzing again to find more issues

## Integrating with Your Own Projects

This tutorial used a GPS parser as an example, but the real power is integrating fuzzing with **your own projects**. 

### Key Integration Steps:

1. **Understand your project structure**: Do you build libraries? Have scattered sources? Complex build system?

2. **Choose your integration approach**:
   - **Library-based** (cleanest): If you can build a static library
   - **Direct sources**: Include your .c files directly in the fuzzer build
   - **Mixed**: Combine libraries and additional sources as needed

3. **Follow the detailed guide**: Read `fuzz/INTEGRATION.md` for step-by-step instructions specific to your chosen integration approach and build system.

4. **Customize the fuzz target**: Replace the GPS parser calls with calls to your own functions.

The scaffolding you generated is designed to be **adaptable** - whether you're using Makefile, CMake, or standalone builds, the `fuzz/INTEGRATION.md` guide will walk you through the specific modifications needed for your project.

## Next Steps

- **Read `fuzz/INTEGRATION.md`**: Detailed integration guide for your specific build system
- **Expand Coverage**: Add more fuzzers for different parts of your code
- **CI Integration**: Run fuzzing in your continuous integration pipeline
- **Structured Input**: For complex formats, consider using structure-aware fuzzing
- **Performance Testing**: Use fuzzing to find performance bottlenecks
- **Security Review**: Analyze found crashes for security implications

## Resources

- [AFL++ Documentation](https://aflplus.plus/)
- [libFuzzer Tutorial](https://llvm.org/docs/LibFuzzer.html)
- [HonggFuzz Guide](https://github.com/google/honggfuzz)
- [Mayhem.security Platform](https://mayhem.security/)
- [Fuzzing Best Practices](https://github.com/google/fuzzing)

Happy fuzzing! 🐛🔍
