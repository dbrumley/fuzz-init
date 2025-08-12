# GitHub Actions Workflows

This directory contains the CI/CD workflows for fuzz-init.

## Workflows

### ci.yml (Main CI)
- **Trigger**: On every push to main and on pull requests
- **Purpose**: Basic build validation and quick smoke test
- **What it does**:
  1. Builds fuzz-init
  2. Runs Rust unit tests
  3. Runs Clippy linting
  4. Checks code formatting
  5. Quick smoke test with CPP CMake template (minimal mode)
  6. Tests dev mode validation

This workflow is designed to be fast and catch basic issues quickly.

### release.yml (Release Builds)
- **Trigger**: On version tags (e.g., v1.0.0)
- **Purpose**: Build release binaries for multiple platforms
- **Platforms**: Linux x64, macOS x64, Windows x64

### full-test.yml (Comprehensive Testing)
- **Trigger**: Manual dispatch or weekly schedule (Sunday 2 AM)
- **Purpose**: Full matrix testing of all template combinations
- **Test Matrix**:
  - Languages: C, CPP
  - Integrations: cmake, make
  - Modes: full, minimal
- **Includes**: Testing with all fuzzing engines (libFuzzer, AFL++, HonggFuzz)

## Design Philosophy

1. **Fast CI for PRs**: The main CI workflow is kept minimal to provide quick feedback
2. **Comprehensive testing on schedule**: Full test suite runs weekly or on-demand
3. **Single source of truth**: Removed duplicate workflows (quick-test.yml, test-flag.yml) to avoid confusion

## Running Tests Locally

To run the same tests locally:

```bash
# Quick smoke test (what CI runs)
cargo test
cargo build --release
./target/release/fuzz-init test-cpp --language CPP --integration cmake --minimal
cd test-cpp/fuzz
cmake --preset fuzz-libfuzzer && cmake --build --preset fuzz-libfuzzer

# Full dev mode test
./target/release/fuzz-init --dev-mode --language CPP --integration cmake
```

## Future Improvements

- Add more language templates to test matrix as they're added
- Consider adding performance benchmarks
- Add integration tests with real fuzzing runs