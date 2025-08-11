# {{project_name}} Fuzzing Tutorial

Welcome to {{project_name}}!

## Quick Start

To build the application and all fuzzers:

```
fuzz.sh build
```


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
├── src/                            # Example application
│   ├── mylib.c                     # Library functions
│   └── main.c                      # Example application
├── include/
│   └── mylib.h                     # Public library interface
├── fuzz.sh                         # Build and test driver
├── build/                          # Location for build artifacts
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

- Your main code is built into a library (`lib{{project_name}}.a`)
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
tests is called the **fuzz testsuite**, aka a **fuzz corpus**. The testsuite
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

#include "mylib.h"

int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
    // Example: Process the input data through your library function
    // Note: Ensure data is properly null-terminated if your function expects a string
    if (size > 0) {
        char* null_terminated = (char*)malloc(size + 1);
        if (null_terminated) {
            memcpy(null_terminated, data, size);
            null_terminated[size] = '\\0';
            process(null_terminated);
            free(null_terminated);
        }
    }
    return 0;
}
```

This demonstrates **key fuzzing principles**:

1. **Universal Entry Point**: The `LLVMFuzzerTestOneInput` function is the
   standard interface that works with all fuzzing engines (AFL, libFuzzer,
   HonggFuzz, standalone) 

2. **Input Processing**: Takes raw bytes from the fuzzer and converts them into
   the format your code expects (here, null-terminated strings) 

3. **Target Coverage**: Calls library functions that exercise complex logic
   where bugs might hide 

4. **Error Handling**: Gracefully handles invalid inputs without crashing
   (unless it's a real bug) 

5. **Cross-Fuzzer Compatibility**: This same code works with:
   - **AFL/AFL++**: Coverage-guided fuzzing with persistent mode
   - **libFuzzer**: Built-in Clang fuzzing engine with structure-aware mutations
   - **HonggFuzz**: Alternative feedback-driven fuzzer
   - **Native**:    Regular executable for debugging and manual testing

## Step 2: How Fuzzing Works

This example demonstrates **coverage-guided fuzzing** - the most effective automated bug-finding technique:

### The Fuzzing Loop

1. **Generate Input**: Fuzzer creates test inputs (starting from seed files in `testsuite/`)
2. **Execute Target**: Runs your code with the input, tracking which code paths are hit
3. **Measure Coverage**: Instruments the binary to see what new code was reached
4. **Mutate & Evolve**: Keeps inputs that found new code paths, mutates them to explore further
5. **Detect Crashes**: When bugs cause crashes, the fuzzer saves the crashing input

### Why This Example Works Well

This example is structured to exhibit best practices in testing and fuzzing:
- **Clean Interface**: link against a library, not individual source files
- **Realistic Testing**: Tests the same code your application would use
- **Easy Integration**: Drop new harnesses for testing and fuzzing without
  restructuring your project or build
- **Cross-Fuzzer Support**: A single harness works with AFL, libfuzzer,
  honggfuzz, and uninstrumented native builds. 


**Key Insight**: The fuzzer doesn't need to know about your internal source structure - it just links against your library like any other application would.


## Common Issues and Solutions

### Build Errors

- **Missing headers**: Make sure all dependencies are installed
- **Linker errors**: Check that all required libraries are linked
- **Fuzzer not found**: Install the fuzzing engine or use standalone mode

### Fuzzing Issues

- **No crashes found**: Try more diverse test cases or longer campaigns
- **Fuzzer gets stuck**: Add better seed inputs or improve dictionary
- **Performance issues**: Profile your fuzz target for bottlenecks


## Best Practices

1. **Start Simple**: Begin with basic functionality before complex scenarios
2. **Fast Execution**: Optimize your fuzz target for speed
3. **Good Seeds**: Provide diverse, valid test cases as starting points
4. **Comprehensive Dictionaries**: Include all relevant keywords and values
5. **Monitor Progress**: Check fuzzer statistics regularly
6. **Reproduce Issues**: Always verify crashes in debug builds
7. **Fix and Retest**: After fixing bugs, run fuzzing again to find more issues

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
