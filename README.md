# fuzz-init

A CLI tool for scaffolding fuzzing projects that addresses common developer pain points.

[![Rust](https://img.shields.io/badge/Built%20with-Rust-red?logo=rust)](https://www.rust-lang.org/)

## Common Fuzzing Problems

Setting up fuzzing infrastructure often involves:

- **Build system complexity**: Integrating fuzzers with existing Makefiles, CMake, or custom build systems
- **Fuzzer lock-in**: Code that works with one fuzzer (e.g., libFuzzer) but not others (AFL, HonggFuzz)
- **Sanitizer configuration**: Proper setup of AddressSanitizer, UBSan, and MemorySanitizer
- **Project structure**: Organizing code to separate application logic from fuzzing harnesses
- **Integration overhead**: Adding fuzzing to existing projects without breaking existing builds
- **Documentation gaps**: Missing or outdated guides for different fuzzer/build combinations

## How fuzz-init Works

fuzz-init generates projects using a **universal fuzzing architecture**:

1. **Standard interface**: Uses `LLVMFuzzerTestOneInput` which works with all major fuzzers
2. **Template system**: Embedded templates for C, C++, Rust, Python with configurable build systems
3. **Conditional generation**: Templates adapt based on your choices (fuzzer, build system, minimal/full mode)
4. **Build system integration**: Generates appropriate Makefiles, CMakeLists.txt, or standalone scripts

This approach means you write your fuzzing logic once, and it works with AFL, libFuzzer, HonggFuzz, and other fuzzers without code changes.

## What You Get

### Project Structure

```
project/
├── src/                    # Your application/library code
├── include/               # Headers
├── fuzz/                  # Fuzzing infrastructure
│   ├── src/target.c      # Fuzzing harness (LLVMFuzzerTestOneInput)
│   ├── driver/main.c     # Universal fuzzer driver
│   ├── testsuite/        # Initial test corpus
│   ├── dictionaries/     # Fuzzer dictionaries
│   └── Makefile         # Build system integration
├── test/                 # Unit tests
└── build/               # Build artifacts
```

### Build System Support

- **Makefile**: Traditional make-based builds with fuzzer targets
- **CMake**: Modern CMake integration with proper target dependencies
- **Standalone**: Self-contained scripts for simple projects

### Fuzzer Compatibility

- **libFuzzer**: Clang-based fuzzing with coverage feedback
- **AFL/AFL++**: Industry-standard fuzzing with mutation strategies
- **HonggFuzz**: Alternative fuzzing engine with different trade-offs
- **Standalone**: Binary targets for manual fuzzing or integration

## Usage

### Basic Usage

```bash
# Interactive mode - prompts for options
fuzz-init

# Specify everything upfront
fuzz-init my-parser --language c --fuzzer libfuzzer --integration cmake

# Minimal mode for existing projects
fuzz-init existing-app --language c --minimal --integration make
```

### Adding to Existing Projects

```bash
# Generate just the fuzz/ directory
fuzz-init . --minimal --language c --fuzzer libfuzzer

# Results in fuzz/ with everything needed to start fuzzing
cd fuzz && make libfuzzer
./my-target-libfuzzer testsuite/
```

### Template Development

```bash
# Test all template configurations
fuzz-init --dev-mode --language c

# Continuous testing during development
fuzz-init --dev-mode --language c --watch src/templates/C/
```

## Template System

Templates are defined in `src/templates/` with metadata in `template.toml`:

- **Conditional file generation**: Files included based on fuzzer/build system choices
- **Variable substitution**: Project names, target names, and configuration values
- **File conventions**: Smart defaults for different file types and extensions
- **Integration metadata**: Supported fuzzers, build systems, and their requirements

### Universal Fuzzer Driver

The generated `fuzz/driver/main.c` provides a consistent interface across fuzzers:

```c
// Works with libFuzzer, AFL, HonggFuzz, and others
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Your fuzzing logic here
    return 0;
}
```

### Build System Integration

Generated build files handle:

- Library compilation with appropriate flags
- Fuzzer-specific compilation (sanitizers, instrumentation)
- Test corpus and dictionary management
- Multiple fuzzer target generation

## Development Mode

For template developers and advanced users:

```bash
# Test all configurations (24 for C template)
fuzz-init --dev-mode --language c

# Results in comprehensive testing report:
# - Build success/failure for each combination
# - Timing information
# - Error details for failed builds
# - Success rate statistics
```

### Template Validation

- Tests every fuzzer × integration × mode combination
- Validates build system integration
- Checks fuzzer compatibility
- Identifies missing dependencies

## Installation

### From Source

```bash
git clone https://github.com/dbrumley/fuzz-init
cd fuzz-init
cargo build --release
```

### Dependencies

- **Rust toolchain**: For building fuzz-init
- **clang/clang++**: For libFuzzer support
- **AFL/AFL++**: Optional, for AFL fuzzing
- **HonggFuzz**: Optional, for HonggFuzz mode

### Verification

```bash
fuzz-init --test  # Check what works on your system
```

## Template Capabilities

| Language | Fuzzers                               | Build Systems           | Unit Tests | Docker  | Mayhem  |
| -------- | ------------------------------------- | ----------------------- | ---------- | ------- | ------- |
| **C**    | AFL, libFuzzer, HonggFuzz, Standalone | Make, CMake, Standalone | ✅ 6 tests | ✅      | ✅      |
| **C++**  | AFL, libFuzzer, HonggFuzz, Standalone | Make, Standalone        | ✅ 6 tests | ✅      | ✅      |
| **Rust** | cargo-fuzz, AFL.rs                    | Cargo                   | Planned    | Planned | Planned |

### C/C++ Template Features

- **Universal fuzzer compatibility** - Same code works with all fuzzers
- **Smart library linking** - Intelligent target detection and dependency management
- **Comprehensive testing** - Unit tests validate functionality before fuzzing
- **Production integration** - Docker, Mayhem, CI/CD ready out of the box
- **Complete documentation** - TUTORIAL.md, INTEGRATION.md, and contextual READMEs

### Rust Template Features

- **cargo-fuzz integration** - Native Rust fuzzing with modern toolchain
- **AFL.rs support** - Alternative fuzzing engine option
- **Cargo-native** - Follows Rust ecosystem conventions

## Example Workflow

**1. Create A New Fuzzing Project**

```bash
fuzz-init myapp --language c --fuzzer libfuzzer --integration make
cd myapp
make        # Build the example library
make test   # Run unit tests (6 comprehensive tests)
make fuzz-libfuzzer  # Build fuzz harnesses
./fuzz/secure-parser-libfuzzer fuzz/testsuite/
# INFO: Running with entropic power schedule (0xFF, 100).
# INFO: Seed: 123456789
# INFO: Loaded 1 modules   (8 inline 8-bit counters): 8 [0x10f7fe0, 0x10f7fe8),
# #1      INITED cov: 3 ft: 3 corp: 1/1b exec/s: 0 rss: 26Mb
# #8      NEW    cov: 4 ft: 4 corp: 2/2b lim: 4 exec/s: 0 rss: 26Mb L: 1/1 MS: 1 ChangeBit-
```

**2. Drop into an existing project**

```bash
myapp$ fuzz-init . --minimal --language cpp --fuzzer libfuzzer --integration cmake
```

Then integrate the `fuzz` directory into your overall build.

**3. Start Fuzzing**

```bash
make fuzz-libfuzzer
./fuzz/secure-parser-libfuzzer fuzz/testsuite/
# INFO: Running with entropic power schedule (0xFF, 100).
# INFO: Seed: 123456789
# INFO: Loaded 1 modules   (8 inline 8-bit counters): 8 [0x10f7fe0, 0x10f7fe8),
# #1      INITED cov: 3 ft: 3 corp: 1/1b exec/s: 0 rss: 26Mb
# #8      NEW    cov: 4 ft: 4 corp: 2/2b lim: 4 exec/s: 0 rss: 26Mb L: 1/1 MS: 1 ChangeBit-
```

**4. Scale to Production**

```bash
# Container-based fuzzing
docker build -t secure-parser-fuzz .
docker run secure-parser-fuzz

# Cloud fuzzing with Mayhem
mayhem run .
```

## Advanced Usage

### Custom Templates from GitHub

```bash
# Use organization templates
fuzz-init my-app --template @myorg/custom-fuzzing-template

# Specific repository with integration override
fuzz-init secure-app --template github:security-team/hardened-template --integration cmake
```

### Multi-Language Projects

```bash
# C library with Rust fuzzing harnesses
fuzz-init hybrid-app --language c --integration cmake
# Then add Rust fuzzing separately
fuzz-init hybrid-app-rust --language rust --minimal
```

### Testing Template Modifications

```bash
# Edit templates in src/templates/
# Test immediately without rebuilding
cargo run -- --dev-mode --language c --fuzzer libfuzzer --dev-output ./test-workspace/
```

## What You Get

Every generated project includes:

- **📖 TUTORIAL.md**: Complete fuzzing tutorial with real examples
- **🔧 INTEGRATION.md**: Step-by-step integration guide for existing projects
- **⚡ README.md**: Quick reference with copy-paste commands
- **🧪 Unit Tests**: Comprehensive test coverage validating functionality
- **🐳 Docker**: Container setup for consistent fuzzing environments
- **☁️ Mayhem**: Cloud fuzzing configuration for production scale
- **📁 Project Structure**: Professional organization following industry best practices

## Common Issues

### Build Failures

- **Missing clang**: Install clang with libFuzzer support
- **AFL not found**: Install AFL++ and ensure it's in PATH
- **Sanitizer errors**: Check compiler version and flags

### Template Issues

- **No templates found**: Ensure templates are embedded in release builds
- **Remote template fails**: Check GitHub repository access and structure

### Integration Problems

- **Build system conflicts**: Use minimal mode for existing projects
- **Library linking issues**: Check target detection in generated build files

## Support & Development

- **🐛 Issues**: Report bugs at [GitHub Issues](https://github.com/dbrumley/fuzz-init/issues)
- **💡 Feature Requests**: We welcome community input on new templates and integrations
- **🤝 Contributing**: See `CONTRIBUTING.md` for development workflow and template creation guide
- **📖 Documentation**: Comprehensive docs generated from CLI definitions

---

**Ready to start fuzzing?** `fuzz-init my-app --language c` and begin in under a minute.
