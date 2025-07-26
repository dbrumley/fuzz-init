# fuzz-init

The easiest way to get started integrating fuzzing into your app is by using
`fuzz-init`. This CLI tool enables you to quickly start building the proper
scaffolding and new fuzz harness, with everything set up for you to run in
Mayhem. You can also create a new skeleton template for a new app that includes
fuzzing and follows best practices.

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
fuzz-init my-app --language c  --integration make --fuzzer libfuzzer
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

- `--template <name>` - Select a specific template (c, c++, python, rust, or
  github:org/repo)
- `--project <name>` - Specify project name via flag instead of argument
- `--test` - Test all templates and fuzzer combinations on your system

## Available Templates

### C Template (`--template c`)
Full-featured C fuzzing template with universal fuzzer support:
- **Supported Fuzzers**: AFL/AFL++, libFuzzer, HonggFuzz, standalone
- **What you get**: Complete Docker setup, Mayhem configuration, build
  scripts, test suites
- **Universal Design**: Write standard `LLVMFuzzerTestOneInput()` and the
  template handles all fuzzer compatibility

### C++ Template (`--template cpp`)  
C++ fuzzing template with comprehensive tooling:
- **Supported Fuzzers**: AFL/AFL++, libFuzzer, HonggFuzz, standalone
- **What you get**: Full C++ project structure with fuzzing infrastructure

### Python Template (`--template python`)
Basic Python project template:
- **Supported Fuzzers**: Standalone (simple project structure)
- **What you get**: Basic Python project scaffold

### Rust Template (`--template rust`)
Rust fuzzing template with cargo integration:
- **Supported Fuzzers**: libFuzzer (via cargo-fuzz), AFL (via afl.rs)
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
   fuzz-init my-fuzz-project --template c
   ```

3. **Build and test your project**:
   ```bash
   cd my-fuzz-project/fuzz
   ./build.sh
   echo "test input" | ./bin/my-fuzz-project
   ```

4. **Try different fuzzer modes**:
   ```bash
   FUZZER_TYPE=standalone ./build.sh    # No dependencies needed
   FUZZER_TYPE=libfuzzer ./build.sh     # If you have libFuzzer
   FUZZER_TYPE=afl ./build.sh           # If you have AFL installed
   ```

The generated templates include detailed README files with specific
instructions for each fuzzer type.

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
