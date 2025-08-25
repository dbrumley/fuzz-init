# fuzz-init

A CLI tool for scaffolding fuzzing projects that addresses common developer pain points.

> ... fuzzing or fuzz testing is an automated software testing technique that involves providing invalid, unexpected,
or random data as inputs to a computer program. &#91;[Wikipedia](https://en.wikipedia.org/wiki/Fuzzing)&#93;

[![Rust](https://img.shields.io/badge/Built%20with-Rust-red?logo=rust)](https://www.rust-lang.org/)
[![CI](https://github.com/dbrumley/fuzz-init/workflows/CI/badge.svg)](https://github.com/dbrumley/fuzz-init/actions)
[![Quick Test](https://github.com/dbrumley/fuzz-init/workflows/Quick%20Test/badge.svg)](https://github.com/dbrumley/fuzz-init/actions)

**Ready to start fuzzing?** `fuzz-init my-app --language c` and begin in under a minute.

## Common Fuzzing Problems

Setting up fuzzing infrastructure often involves:

- **Build system complexity**: Integrating fuzzers with existing build like
  `make`, `cmake`, `cargo`, and so on. 
- **Fuzzer lock-in**: Code that works with one fuzzer (e.g., libFuzzer) but not
  others (AFL, HonggFuzz)
- **Sanitizer configuration**: Proper setup of AddressSanitizer, UBSan, and
  MemorySanitizer
- **Project structure**: Organizing code to separate application logic from
  fuzzing harnesses
- **Integration overhead**: Adding fuzzing to existing projects without
  breaking existing builds
- **Documentation gaps**: Missing or outdated guides for different fuzzer/build
  combinations
- **Onboarding gaps**: Bringing new developers up to speed on how fuzzing works

## Installation

### From GitHub Releases (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/dbrumley/fuzz-init/releases):

```bash
# Linux
wget https://github.com/dbrumley/fuzz-init/releases/latest/download/fuzz-init-linux-x64
chmod +x fuzz-init-linux-x64
./fuzz-init-linux-x64 --help

# macOS
wget https://github.com/dbrumley/fuzz-init/releases/latest/download/fuzz-init-macos-x64
chmod +x fuzz-init-macos-x64
./fuzz-init-macos-x64 --help

# Windows
# Download fuzz-init-windows-x64.exe from the releases page
```

### From Source

```bash
git clone https://github.com/dbrumley/fuzz-init
cd fuzz-init
./install.sh
```

#### Dependencies

- **Rust toolchain**: For building fuzz-init
- **clang/clang++**: For libFuzzer support
- **AFL/AFL++**: Optional, for AFL fuzzing
- **HonggFuzz**: Optional, for HonggFuzz mode

## Usage

### Basic Usage

```bash
# Interactive mode - prompts for options
fuzz-init

# Specify language and integration up front
fuzz-init my-parser --language cpp --integration cmake

# Don't include the tutorial and sample application
fuzz-init existing-app --language cpp --integration make --minimal 

# Run in dev-mode to test a template for a particular language
fuzz-init  --language cpp --dev-mode --integration cmake  --dev-output ./scratch/

# Same as above, but rebuild when the template changes.
fuzz-init  --language cpp --dev-mode --integration cmake  --dev-output ./scratch/ --watch
```

### Adding to Existing Projects

```bash
# Generate just the fuzz/ directory
fuzz-init . --minimal --language cpp

# Results in fuzz/ with everything needed to start fuzzing
cd fuzz && make libfuzzer
./my-target-libfuzzer testsuite/
```

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

## How fuzz-init Works

`fuzz-init` generates projects using a **universal fuzzing architecture**:

1. **Standard interface**: Standard `LLVMFuzzerTestOneInput` entrypoint
   configured to work with all major fuzzers. 
2. **Template system**: Embedded templates for C, C++, Rust, and the ability to
   extend with remote templates. 
3. **Conditional generation**: Templates adapt based on your choices (set
   default fuzzer, build system, minimal/full mode)
4. **Drop-in or tutorial-based**: Generate just fuzz templates, or a full
   tutorials to speed up onboarding.

## What You Get

Every generated project includes:

- **📖 TUTORIAL.md**: Complete fuzzing tutorial with real examples
- **🔧 INTEGRATION.md**: Step-by-step integration guide for existing projects
- **⚡ README.md**: Quick reference with copy-paste commands
- **🧪 Unit Tests**: Comprehensive test coverage validating functionality
- **🐳 Docker**: Container setup for consistent fuzzing environments
- **☁️ Mayhem**: Cloud fuzzing configuration for production scale
- **📁 Project Structure**: Professional organization following industry best practices

### Project Structure

```
project/
├── src/                  # Your application/library code
├── include/              # Headers
├── fuzz/                 # Fuzzing infrastructure
│   ├── src/{a,b,c}.c     # Fuzzing harnesses (LLVMFuzzerTestOneInput)
│   ├── driver/main.c     # Universal fuzzer driver
│   ├── testsuite/        # Initial test corpus
│   ├── dictionaries/     # Fuzzer dictionaries
├── test/                 # Unit tests
└── build/               # Build artifacts, including executable fuzzers.
```

### Fuzzer Compatibility

A new project includes rules for building as many fuzzers as possible so you
don't have to guess which will be best. Our structure enables all fuzzers
supported by the particular language, including: 

- [**libFuzzer**](https://llvm.org/docs/LibFuzzer.html): Clang-based fuzzing with coverage feedback
- [**AFL/AFL++**](https://aflplus.plus/): Industry-standard fuzzing with mutation strategies
- [**HonggFuzz**](https://honggfuzz.dev/): Alternative fuzzing engine with different trade-offs
- **Native**: Binary targets for manual fuzzing or integration

**Example:** Using `cmake` with clang, you'd do:

```bash
fuzz-init myapp --language cpp --integration cmake
cd myapp
./fuzz.sh  # See how to build and run fuzzers configured for cmake.
```

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

## Template System

Templates are defined in `src/templates/` with metadata in `template.toml`:

- **Conditional file generation**: Files included based on fuzzer/build system choices
- **Variable substitution**: Project names, target names, and configuration values
- **File conventions**: Smart defaults for different file types and extensions
- **Integration metadata**: Supported fuzzers, build systems, and their requirements

Learn more about templates in the [Template System Guide](./TEMPLATING.md).

### Template Development

```bash
# Test all template configurations
fuzz-init --dev-mode --language c

# Continuous testing during development
fuzz-init cargo run -- --dev-mode --language CPP --watch --dev-output ./scratch/
```

### Testing Template Modifications

```bash
# Edit templates in src/templates/
# Test immediately without rebuilding
cargo run -- --dev-mode --language c --fuzzer libfuzzer --dev-output ./test-workspace/
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

## Support & Development

- **🐛 Issues**: Report bugs at [GitHub Issues](https://github.com/dbrumley/fuzz-init/issues)
- **💡 Feature Requests**: We welcome community input on new templates and integrations
- **🤝 Contributing**: See the [contribution guide](./CONTRIBUTING.md) to learn more about the development workflow and
  template creation
- **📖 Documentation**: Comprehensive docs generated from CLI definitions

---

[![Star History Chart](https://api.star-history.com/svg?repos=dbrumley/fuzz-init&type=Date)](https://star-history.com/#dbrumley/fuzz-init&Date)
