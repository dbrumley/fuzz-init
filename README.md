# fuzz-init

The easiest way to get started integrating fuzzing into your app is by using
`fuzz-init`. This CLI tool enables you to quickly start building the proper
scaffolding and new fuzz harness, with everything set up for you to run with
fuzzers like AFL, libfuzzer, HonggFuzz, and Mayhem. You can also create a new skeleton template for a new app that includes
fuzzing, unit testing, and follows best practices.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/fuzz-init
cd fuzz-init

# Option 1: Using make
make install

# Option 2: Using install script
./install.sh

# Option 3: Manual installation
cargo build --release
sudo cp target/release/fuzz-init /usr/local/bin/
```

### Verify Installation

```bash
fuzz-init --help
```

## Quick Start

To get started, simply run:

```bash
fuzz-init
```

This will prompt you to:
1. Enter a project name
2. Choose a language template (C, C++, Python, Rust)
3. Select your preferred fuzzer (AFL, libFuzzer, HonggFuzz, or standalone)
4. Select your preferred build system (make, cmake, cargo, etc)


To create scaffolding in a specific folder with all options specified:

```bash
fuzz-init myapp --language c --integration make --fuzzer libfuzzer
```

To create a minimal fuzz directory for integration with existing projects:

```bash
fuzz-init myapp --language c --minimal --integration make --fuzzer libfuzzer
```

To create a project from a GitHub template:

```bash
fuzz-init myapp --template github:user/repo
fuzz-init myapp --template @user/repo  # shorthand syntax
```

## Testing Your Setup

**New!** Before diving into fuzzing, you can verify that all templates work
correctly on your system:

```bash
fuzz-init --test
```

This will:
- ✅ Test all available templates (C, C++, Python, Rust)
- ✅ Try building with every fuzzer option (AFL, libFuzzer, HonggFuzz,
  standalone)
- ✅ Show you exactly which combinations work on your system
- ✅ Identify missing dependencies or configuration issues

### Example Test Output

```
🧪 Running template tests...

Testing template: c
  Fuzzer options: afl, libfuzzer, hongfuzz, standalone
  Testing fuzzer: afl
    ❌ Build failed - AFL not properly configured
  Testing fuzzer: libfuzzer
    ✅ Build successful
  Testing fuzzer: standalone
    ✅ Build successful

📊 Test Summary:
================
❌ c - 2/4 fuzzer modes passed
   └─ ❌ afl failed
✅ python - All 1 fuzzer modes passed
```

**Why test?** Fuzzing tools like AFL and HonggFuzz require specific
installations and configurations. The test mode helps you:
- Verify your fuzzer installations work correctly
- Identify which templates are ready to use
- Debug setup issues before starting a project
- Ensure templates will build successfully

## Options

`fuzz-init` comes with the following options:

- `--language <name>` - Select a programming language (c, cpp, python, rust)
- `--fuzzer <name>` - Select fuzzer (afl, libfuzzer, honggfuzz, standalone)
- `--integration <type>` - Select build system (standalone, make, cmake)
- `--minimal` - Generate minimal fuzz directory for existing projects
- `--template <name>` - Use remote template (github:org/repo or @org/repo)
- `--test` - Test all templates and fuzzer combinations on your system

## Available Templates

### C Template (`--language c`)
Full-featured C fuzzing template with universal fuzzer support:
- **Supported Fuzzers**: AFL/AFL++, libFuzzer, HonggFuzz, standalone
- **Build Systems**: Makefile, CMake, standalone build script
- **What you get**: Complete project with library, unit tests, Docker setup, Mayhem configuration, comprehensive documentation
- **Universal Design**: Write standard `LLVMFuzzerTestOneInput()` and the template handles all fuzzer compatibility
- **Testing**: Comprehensive unit test suite with 6 tests covering all library functions
- **Integration Modes**: Full tutorial mode or minimal mode for existing projects

### C++ Template (`--language cpp`)  
C++ fuzzing template with comprehensive tooling:
- **Supported Fuzzers**: AFL/AFL++, libFuzzer, HonggFuzz, standalone
- **Build Systems**: Makefile, standalone build script
- **What you get**: Full C++ project structure with fuzzing infrastructure

### Python Template (`--language python`)
Basic Python project template:
- **Supported Fuzzers**: Standalone (simple project structure)
- **Build Systems**: Standalone
- **What you get**: Basic Python project scaffold

### Rust Template (`--language rust`)
Rust fuzzing template with cargo integration:
- **Supported Fuzzers**: libFuzzer (via cargo-fuzz), AFL (via afl.rs)
- **Build Systems**: Standalone (cargo-based)
- **What you get**: Rust project configured for modern Rust fuzzing tools

### Remote Templates
You can also use templates from GitHub repositories:
```bash
fuzz-init my-project --template github:forallsecure/c-template
fuzz-init my-project --template @forallsecure/c-template  # short
```

## Prerequisites

### For C/C++ Templates
- **clang** - For libFuzzer and standalone builds
- **AFL/AFL++** - Optional, for AFL fuzzing mode
- **HonggFuzz** - Optional, for HonggFuzz mode

### For Rust Template  
- **cargo-fuzz** - For libFuzzer integration
- **afl.rs** - Optional, for AFL integration

### Installation Check
The easiest way to see what's working on your system:
```bash
fuzz-init --test
```

## Getting Started Example

1. **Test your setup first**:
   ```bash
   fuzz-init --test
   ```

2. **Create a new C fuzzing project**:
   ```bash
   fuzz-init my-fuzz-project --language c --fuzzer libfuzzer --integration make
   ```

3. **Build library and run unit tests**:
   ```bash
   cd my-fuzz-project
   make lib          # Build the library
   make test         # Run unit tests
   ```

4. **Build and test the fuzzer**:
   ```bash
   make fuzz-libfuzzer                    # Build fuzzer
   cd fuzz && ./build/fuzz/my-fuzz-project-libfuzzer testsuite/
   ```

5. **Try different build systems**:
   ```bash
   # CMake integration
   fuzz-init cmake-project --language c --integration cmake --fuzzer libfuzzer
   cd cmake-project && mkdir build && cd build
   CC=clang cmake .. && cmake --build . --target test
   
   # Minimal mode for existing projects
   fuzz-init existing-project --language c --minimal --integration make
   ```

The generated templates include detailed documentation:
- **TUTORIAL.md**: Complete fuzzing tutorial with examples
- **fuzz/INTEGRATION.md**: Integration guide for existing projects  
- **fuzz/README.md**: Quick reference for fuzzing commands

## Development Environment

### Using VS Code Dev Container (Recommended)

For the best development experience with all fuzzing tools pre-installed:

1. **Prerequisites**: Install VS Code with the Dev Containers extension
2. **Open project**: VS Code will prompt to "Reopen in Container"
3. **Ready to go**: All fuzzing tools (AFL++, HonggFuzz, libFuzzer) work
   out-of-the-box in Linux

Benefits:
- ✅ No macOS compatibility issues
- ✅ All fuzzing tools pre-installed and configured  
- ✅ Consistent environment across development machines
- ✅ Perfect for running `fuzz-init --test`

### Manual Setup

If not using the devcontainer, install these tools for full functionality:
- **clang/clang++** - For libFuzzer and standalone builds
- **AFL++** - `git clone https://github.com/AFLplusplus/AFLplusplus`
- **HonggFuzz** - `git clone https://github.com/google/honggfuzz`
- **Rust & cargo-fuzz** - `cargo install cargo-fuzz`
