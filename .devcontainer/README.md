# Mayhem Init Development Container

This devcontainer provides a complete Linux environment with all fuzzing tools
pre-installed and configured for mayhem-init development and testing.

## What's Included

- **Rust toolchain** - Latest stable Rust with cargo
- **clang/clang++** - LLVM compiler with libFuzzer support
- **AFL++** - Advanced fuzzing framework  
- **HonggFuzz** - Google's coverage-guided fuzzer
- **cargo-fuzz** - Rust fuzzing integration
- **Python 3** - With atheris fuzzing library
- **VS Code extensions** - Rust analyzer, C++ tools, Python support

## Quick Start

1. **Open in VS Code**: Click "Reopen in Container" when prompted, or use
   Command Palette > "Dev Containers: Reopen in Container"

2. **Verify setup**: The container will show tool versions on startup

3. **Run tests**: All fuzzing tools should work out-of-the-box
   ```bash
   cargo run -- --test
   ```

## Benefits

- **Consistent environment** - Same setup across all development machines
- **No macOS issues** - Linux environment where fuzzing tools work reliably  
- **Pre-configured** - All dependencies installed and working
- **Isolated** - No impact on host system

## Development Workflow

```bash
# Build the project
cargo build

# Run comprehensive tests
cargo run -- --test

# Create test projects
cargo run -- my-test --template c

# Test specific fuzzer
cd my-test/fuzz
FUZZER_TYPE=afl ./build.sh
```

All fuzzing tools (AFL++, HonggFuzz, libFuzzer, standalone) should work
perfectly in this environment.