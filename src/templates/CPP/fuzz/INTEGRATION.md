# Integrating {{project_name}} Fuzzing into Your Project

`fuzz-init` scaffolds fuzzing using a universal fuzzing design - write once,
fuzz everywhere! The same `LLVMFuzzerTestOneInput()` function works with AFL,
libFuzzer, HonggFuzz, and standalone testing.

## Quickstart

 - Verify everything works with  `fuzz.sh build && fuzz.sh test`
 - Fuzz targets are under the `build` directory.
 - **Important!:** Integrate this directory into your projects overall build.
 - Modify `src/fuzz_harness_1.cpp` to fuzz your project code.
 - Rebuild, fuzz, and find bugs!

## Integration Overview

There are four main steps to integrating this scaffolding into your project:

1. **Integrate**: Add this directory (`fuzz`) so that it's built with your
   entire project.
2. **Compilers & Instrumentation (Recommended)**: Set up a built target with
   the appropriate instrumentation and sanitizer flags.
3. **Harness**: Write fuzz harnesses that target application logic on your
   apps attack surface.
4. **Fuzz**: Perform spot fuzzing locally, or integrate into a commercial full
   CICD pipeline solution like [Mayhem](https://mayhem.security).


## Build Integration

The `fuzz` directory should be integrated into your overall project so that
your fuzz harnesses stay in sync with your source code. We have provided a
skeleton for several build/integration systems and fuzzing engines.

### Fuzzing Engines

The script `./fuzz.sh` shows how to build this repo for several different
fuzzing engines.  We have standardized harnesses around
`LLVMFuzzerTestOneInput` used by libfuzzer, and have included drivers for
other fuzzers to use this as the harness entrypoint.

 - Out-of-the-box this will build for all fuzzers installed on your system.
 - You can do native builds, choose one fuzzing engine, or build for multiple
   -- it's up to you.
 - **Picking a fuzzing engine**:
    - **AFL** is the best choice for applications.
        - AFL requires linking against a `main` function. We have provided a
          universal AFL harness driver under `driver/main.cpp`
        - Install with `apt-get install afl++`.
        - Set your C++ compiler to `afl-clang-fast++` or `afl-clang-g++`.
        - You can separately enable sanitizers.
    - **libfuzzer** is the best choice for pure libraries.
        - libfuzzer does not need a driver and uses it's own `main` function.
        - It uses the same process for each fuzzing iteration to improve
          speed, but this also means memory leaks accumulate and can
          cause an OOM (out-of-memory) error.
        - It requires clang. Install with `apt-get install clang`
    - **honggfuzz** is an alternative to AFL++. See `Dockerfile` for an
      example installation of **honggfuzz**.
    - [Mayhem](https://mayhem.security) is the most comprehensive, and allows
      for native (uninstrumented) fuzzing. It also supports running AFL,
      libfuzzer, and honggfuzz targets. Get a free trial at
      [https://app.mayhem.security].


{{#if (eq integration 'cmake')}}
### CMake Integration

Start by adding this directory (`fuzz`) to your top-level `CMakeLists.txt`
file:
```Makefile
# Add to your CMakeLists.txt
add_subdirectory(fuzz)
```

Then update the `fuzz/CMakeLists.txt` file to pass proper include and linker
flags to your harnesses:
```Makefile
# Use any project-level headers
target_include_directories(${FUZZ_EXE} PRIVATE ${CMAKE_SOURCE_DIR}/include)
# Link against any project-level libraries.
target_link_libraries(${FUZZ_EXE} PRIVATE mylib)
```

The `fuzz-init` scaffolding also sets up `cmake` presets.

   ```bash
   $ cmake --list-presets
      Available configure presets:

      "base"            - Base
      "fuzz-standalone" - Fuzz (standalone)
      "fuzz-libfuzzer"  - Fuzz (libFuzzer)
      "fuzz-afl"        - Fuzz (AFL++)
      "fuzz-honggfuzz"  - Fuzz (Honggfuzz)

   # Build libfuzzer targets
   cmake --preset fuzz-libfuzzer && cmake --build --preset fuzz-libfuzzer

   # Similar for other targets
   ```


**Note:** Unfortunately addressing every possible build configuration is out
of scope for this tool; please see the `cmake` documentation.

**Tips:**
- **Important!** You need to compile your *entire* project with instrumentation
   and sanitizers, not just this `fuzz` directory.
- Common compiler settings can be found under the `cmake/` directory.
- `CMakePresets.json` gives example recipies for using the above `cmake`
  directives.
- AFL++ requires the entire project be compiled with `afl-{g,clang}++` compiler
  (`apt-get install afl++`)
- libfuzzer requires the `clang` compiler (`apt-get install clang`)
- If you want a complete application example, run `fuzz-init` without the
  `--minimal` setting to get a full project with tutorial.

{{/if}}

{{#if (eq integration 'make')}}

### Makefile integration

To integrate, start by adding this directory to your `Makefile` as a new target:
```Makefile
# Build fuzz targets (delegates to fuzz/Makefile)
fuzz: $(LIBRARY)
	@echo "🔨 Building fuzzing targets..."
	@$(MAKE) -C fuzz all
	@echo "🎯 Fuzzing build complete!"

# Per-fuzzer targets that rebuild library with appropriate compiler/flags
fuzz-libfuzzer:
	@echo "🔨 Building library and fuzz targets for libFuzzer..."
	@if [ "$(HAVE_CLANG)" = "yes" ]; then \
	  $(MAKE) clean-lib && \
	  $(MAKE) lib CXX=clang++ CXXFLAGS="-g -O1 -fsanitize=address,undefined -I$(INC_DIR) -std=c++17" && \
	  $(MAKE) -C fuzz libfuzzer LIBPART=../$(LIBRARY) CXX_CLANG=clang++; \
	else \
	  echo "⏭️  libFuzzer requires clang++"; \
	fi
```

We provide `fuzz.sh`, but you can also directly use `make`:
   ```bash
   # For libFuzzer
   make clean
   CXX=clang++ CXXFLAGS="-fsanitize=address,undefined -g -O1" make

   # For AFL++
   make clean
   CXX=afl-clang-fast++ CXXFLAGS="-fsanitize=address,undefined -g -O1" make

   # Similar for other fuzzers...
   ```


**Note:** Unfortunately addressing every possible build configuration is out
of scope for this tool; please see the `make` documentation.

**Tips:**
- **Important!** You need to compile your *entire* project with instrumentation
   and sanitizers, not just this `fuzz` directory.
- AFL++ requires the entire project be compiled with `afl-{g,clang}++` compiler
  (`apt-get install afl++`)
- libfuzzer requires the `clang` compiler (`apt-get install clang`)
- If you want a complete application example, run `fuzz-init` without the
  `--minimal` setting to get a full project with tutorial.

{{/if}}

## Scaffolding File Structure

```bash
.
├── CMakeLists.txt       # (cmake only) cmake build directives
├── CMakePresets.json    # (cmake only) build prefixes; see cmake --list-presets
├── fuzz.sh              # helper utility to build and test fuzzers
├── INTEGRATION.md       # this document
├── Mayhemfile           # Template Mayhemfile
├── build                # Build output
│   ├── afl              # AFL compiled targets. Requires afl package
│   ├── honggfuzz         # Honggfuzz compiled targets. Requires honggfuzz
│   ├── libfuzzer        # libfuzzer compiled targets. Requires clang
│   └── standalone       # uninstrumented targets. Native compilation.
├── cmake                # (cmake only) cmake directives for each fuzzer
│   ├── afl.cmake
│   ├── honggfuzz.cmake
│   ├── libfuzzer.cmake
│   └── standalone.cmake
├── dictionaries         # (Optional) fuzz dictionary location
│   └── fuzz_harness_1.dict
├── driver
│   └── main.cpp         # AFL/native main() driver for targets
├── src                  # Standard location for all fuzz harnesses
│   └── fuzz_harness_1.cpp # A single harness
└── testsuite            # Standard location for fuzz testsuites (corpus)
    └── fuzz_harness_1   # test suite for a single harness
        ├── demo_crash.txt  # test file
        └── safe.txt        # test file
```

## Universal Harness and Best Practices

This project uses a universal harness design that works across all major
fuzzers.  Each harness is placed under `src` and must implement
`LLVMFuzzerTestOneInput` that calls your code to test.

  - Place harnesses under `src`, using one file per harness.
  - Place the starting test suite/corpus for the harness under
    `testsuite/<harness name>`.
  - (Optional) Dictionaries can significanty improve anything dealing with
    text. Place dictionaries under `dictionaries/<harness name>.dict`.
  - Set up your main project to build most files into a library. This will
    make testing -- not just fuzzing -- much easier to manage by simplifying
    includes and linking.



## Testing your compiled fuzz targets

Fuzz targets live under the `build` directory, and are organized by fuzzer.
For example:

```bash
# Test that your libfuzzer target harness works
echo "test" | ./build/libfuzzer/bin/fuzz_harness_1-libfuzzer

# Run libfuzzer target harness for 60 seconds
./build/libfuzzer/bin/fuzz_harness_1-libfuzzer -max_total_time=60

# Run AFL for 10 seconds using the testsuite and putting results in 'out'
mkdir out
afl-fuzz -i testsuite/fuzz_harness_1/ -o out -V 10 -- build/afl/bin/fuzz_harness_1-afl
```


## Best Practices

### 1. Consistent Sanitizer Usage

**The library and fuzzer MUST use the same sanitizers**, otherwise you'll get linker errors or miss bugs:


{{#if minimal}}
- When building for libFuzzer/AFL/HonggFuzz: Use `-fsanitize=address,undefined`
- When building for standalone: No sanitizers needed
- Each fuzzer requires rebuilding your application with its specific compiler
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

### 3. Focus on functions that accept untrusted user input

Vulnerabilities typically occur when an applicaton first parses user input, so
those should be the highest priority to fuzz. Typically a pentester will look
for:
  - Functions with `parse` in their name.
  - Functions that accept `char *`
  - Functions that read from a socket or file.

### 4. Seed Test Suite (aka Corpus)

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

🚀 Happy Fuzzing!
