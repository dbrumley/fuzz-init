# {{project_name}} Fuzzing Tutorial

Welcome to the complete fuzzing tutorial for {{project_name}}! This guide will walk you through the entire process of setting up and running fuzzing campaigns using multiple fuzzing engines.

## What is Fuzzing?

Fuzzing is an automated software testing technique that finds bugs by providing unexpected or random data as input to a program. Modern fuzzers like AFL, libFuzzer, and HonggFuzz use sophisticated techniques like genetic algorithms and code coverage feedback to efficiently explore program execution paths.

## Project Structure

Your fuzzing project has been set up with the following structure:

```
{{project_name}}/
├── fuzz/                           # All fuzzing-related files
│   ├── src/{{target_name}}.c      # Your fuzz target implementation
│   ├── driver/main.c               # Universal fuzzer driver
│   ├── build.sh                    # Build script for all fuzzers
│   ├── testcases/                  # Initial test inputs
│   ├── dictionaries/               # Fuzzing dictionaries
│   ├── Dockerfile                  # Container for reproducible fuzzing
│   ├── Mayhemfile                  # Mayhem.security configuration
│   └── README.md                   # Quick reference guide
└── TUTORIAL.md                     # This comprehensive tutorial
```

## Step 1: Understanding Your Fuzz Target

Open `fuzz/src/{{target_name}}.c` and examine the fuzz target:

```c
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    if (size > 4 && data[0] == 'F' && data[1] == 'U' && data[2] == 'Z' && data[3] == 'Z') {
        printf("Boom!\\n");
    }
    return 0;
}
```

This is a universal fuzz target that works with all supported fuzzing engines:
- **AFL/AFL++**: Uses persistent mode for efficiency
- **libFuzzer**: Built-in fuzzing engine in Clang
- **HonggFuzz**: Alternative coverage-guided fuzzer
- **Standalone**: Regular executable for manual testing

## Step 2: Customize Your Fuzz Target

Replace the example code with your actual testing logic:

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

## Step 3: Build and Test

The universal build system supports multiple fuzzing engines:

```bash
cd fuzz/

# Build standalone version (no fuzzer required)
./build.sh
./{{target_name}}-standalone testcases/

# Build with AFL (requires AFL++ installation)
USE_AFL=1 ./build.sh
afl-fuzz -i testcases -o findings -- ./{{target_name}}-afl

# Build with libFuzzer (requires Clang)
USE_LIBFUZZER=1 ./build.sh
./{{target_name}}-libfuzzer -dict=dictionaries/{{target_name}}.dict testcases/

# Build with HonggFuzz (requires HonggFuzz installation)
USE_HONGGFUZZ=1 ./build.sh
honggfuzz -i testcases -W corpus -- ./{{target_name}}-honggfuzz
```

## Step 4: Customize Test Cases and Dictionaries

### Test Cases
Replace the example test cases in `testcases/` with inputs relevant to your code:

```bash
cd testcases/
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
afl-fuzz -i testcases -o findings -M main -- ./{{target_name}}-afl

# Terminal 2: Secondary fuzzer
afl-fuzz -i testcases -o findings -S secondary -- ./{{target_name}}-afl

# Check status
afl-whatsup findings/
```

### libFuzzer Campaign
```bash
mkdir corpus
./{{target_name}}-libfuzzer corpus/ testcases/ \\
    -dict=dictionaries/{{target_name}}.dict \\
    -jobs=4 \\
    -workers=4 \\
    -max_total_time=3600
```

### HonggFuzz Campaign
```bash
honggfuzz \\
    -i testcases \\
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

## Next Steps

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