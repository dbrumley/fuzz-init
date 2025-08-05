# fuzz-init

**Automated scaffolding to get you fuzzing and finding bugs quickly.**

[![Rust](https://img.shields.io/badge/Built%20with-Rust-red?logo=rust)](https://www.rust-lang.org/)
[![Universal Fuzzing](https://img.shields.io/badge/Universal-Fuzzing-blue)]()
[![Build Systems](https://img.shields.io/badge/Makefile%20%7C%20CMake%20%7C%20Cargo-Integrated-green)]()

`fuzz-init` is a CLI tool that scaffolds production-ready fuzzing projects with
enterprise-grade templates.

## ⚡ Why fuzz-init?

**Write Once, Fuzz Everywhere**: Your code works with AFL, libFuzzer,
HonggFuzz, and Mayhem without changes. We handle the complexity.

**Production-Ready Templates**: Complete projects with unit tests, Docker
containers, CI/CD integration, and comprehensive documentation—not toy
examples.

**Zero Configuration**: Automatic compiler detection, intelligent library
linking, and build system integration. No manual environment setup required.

**Professional Workflow**: Full tutorial mode for learning or minimal mode for
existing projects. Your choice.

**Extend Easily**: Create and use your own templates.

## 🚀 Quick Start

```bash
# Get up and running in 30 seconds
fuzz-init my-app --language c --fuzzer libfuzzer

cd my-app
make lib && make fuzz-libfuzzer
./fuzz/my-app-libfuzzer fuzz/testsuite/
```

**That's it.** You're now fuzzing with libFuzzer, complete with sanitizers,
dictionaries, and a working harness.

## ✨ Key Features

### 🎯 **Universal Fuzzing Architecture**

Write standard `LLVMFuzzerTestOneInput()` and we make it work with every
fuzzer. Works on Linux, macOS, containers, and CI?CD systems.

### 🏗️ **Enterprise-Grade Templates**

- **Complete projects**: Library builds, unit tests, Docker containers, Mayhem
  integration
- **Multiple build systems**: Makefile, CMake, and standalone options with
  intelligent linking
- **Library-first design**: Clean separation between your code and fuzzing
  infrastructure

### 📦 **Flexible Integration**

- **Full mode**: Complete tutorial project with examples and documentation
- **Minimal mode**: Just the fuzz directory for existing projects
- **Remote templates**: Use GitHub repos as templates with `@org/repo` syntax

### 🧪 **Advanced Development Tools**

- **Template development mode**: Test all 24+ configurations with `--dev-mode`
- **Real-time feedback**: Debug builds load templates from filesystem—edit and test instantly
- **Comprehensive testing**: Validate your entire fuzzing setup with `--test`

## 💡 Installation

### Quick Install (Recommended)

```bash
# Clone and install
git clone https://github.com/dbrumley/fuzz-init
cd fuzz-init && make install
```

### Verify Setup

```bash
fuzz-init --test  # Check what fuzzing tools work on your system
```

## 🛠️ Usage Patterns

### For New Projects

```bash
# Interactive mode - prompts for all options
fuzz-init

# Specify everything upfront
fuzz-init my-parser --language c --fuzzer libfuzzer --integration cmake

# Use remote template
fuzz-init secure-app --template @forallsecure/c-template
```

### For Existing Projects

```bash
# Add fuzzing to existing codebase
fuzz-init existing-app --language c --minimal --integration makefile

# Results in: existing-app/fuzz/ with everything ready to build
```

### For Template Developers

```bash
# Test all configurations (24 for C template)
fuzz-init --dev-mode --language c

# Focus on specific combination
fuzz-init --dev-mode --language c --fuzzer libfuzzer --integration cmake

# Continuous development with file watching
fuzz-init --dev-mode --language c --watch src/templates/C/
```

## 📊 Template Capabilities

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

## 🎓 Example Workflow

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

**4. Start Fuzzing**

```bash
make fuzz-libfuzzer
./fuzz/secure-parser-libfuzzer fuzz/testsuite/
# INFO: Running with entropic power schedule (0xFF, 100).
# INFO: Seed: 123456789
# INFO: Loaded 1 modules   (8 inline 8-bit counters): 8 [0x10f7fe0, 0x10f7fe8),
# #1      INITED cov: 3 ft: 3 corp: 1/1b exec/s: 0 rss: 26Mb
# #8      NEW    cov: 4 ft: 4 corp: 2/2b lim: 4 exec/s: 0 rss: 26Mb L: 1/1 MS: 1 ChangeBit-
```

**5. Scale to Production**

```bash
# Container-based fuzzing
docker build -t secure-parser-fuzz .
docker run secure-parser-fuzz

# Cloud fuzzing with Mayhem
mayhem run .
```

## 🔧 Development Mode

Perfect for template developers and advanced users:

```bash
# Test all 24 C template configurations
fuzz-init --dev-mode --language c

🔧 Starting template development mode...
📁 Workspace: /tmp/.tmpABC123

🧪 Testing 24 configurations for C template:
[1/24] Testing: afl + makefile + full
    ✅ afl + makefile + full (1.2s)
[2/24] Testing: libfuzzer + cmake + minimal
    ✅ libfuzzer + cmake + minimal (0.8s)

═══════════════════════════════════════
📊 Test Results Summary
═══════════════════════════════════════
Total:      24
✅ Success: 20
❌ Failed:   4

📈 Success rate: 83.3%
⏱️  Average build time: 1.1s
```

### Development Features

- **Instant iteration**: Debug builds load templates from filesystem—no recompilation needed
- **Comprehensive testing**: Every fuzzer×integration×mode combination
- **Persistent debugging**: Use `--dev-output ./debug/` to preserve failed builds
- **Watch mode**: Continuous testing with file system monitoring

## 🏢 Professional Features

### Enterprise Integration

- **CI/CD Ready**: GitHub Actions, Jenkins, GitLab CI templates included
- **Container-First**: Docker and devcontainer support for consistent environments
- **Mayhem Integration**: Production-ready cloud fuzzing configuration
- **Multiple Build Systems**: Native Makefile, CMake, and standalone support

### Quality Assurance

- **Unit Testing**: Comprehensive test suites validate library functionality
- **Sanitizer Integration**: AddressSanitizer, UBSan, MemorySanitizer configured correctly
- **Cross-Platform**: Linux, macOS, Windows (WSL) support with platform-specific optimizations

### Developer Experience

- **Rich Documentation**: Context-aware guides for every template and integration type
- **Intelligent Defaults**: Smart selections based on your environment and preferences
- **Error Recovery**: Detailed failure reporting with actionable remediation steps

## 🌟 Advanced Usage

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

## 📚 What You Get

Every generated project includes:

- **📖 TUTORIAL.md**: Complete fuzzing tutorial with real examples
- **🔧 INTEGRATION.md**: Step-by-step integration guide for existing projects
- **⚡ README.md**: Quick reference with copy-paste commands
- **🧪 Unit Tests**: Comprehensive test coverage validating functionality
- **🐳 Docker**: Container setup for consistent fuzzing environments
- **☁️ Mayhem**: Cloud fuzzing configuration for production scale
- **📁 Project Structure**: Professional organization following industry best practices

## 🛡️ Security Focus

Built for security professionals who need:

- **Vulnerability Discovery**: Templates optimized for finding real security bugs
- **Sanitizer Integration**: Proper AddressSanitizer, UBSan, MemorySanitizer configuration
- **Corpus Management**: Intelligent test case organization and dictionary generation
- **Scalable Architecture**: From local development to cloud-scale continuous fuzzing

## 📞 Support & Development

- **🐛 Issues**: Report bugs at [GitHub Issues](https://github.com/dbrumley/fuzz-init/issues)
- **💡 Feature Requests**: We welcome community input on new templates and integrations
- **🤝 Contributing**: See `CONTRIBUTING.md` for development workflow and template creation guide
- **📖 Documentation**: Comprehensive docs generated from CLI definitions

---

**Ready to find bugs?** `fuzz-init my-app --language c` and start fuzzing in under a minute.
