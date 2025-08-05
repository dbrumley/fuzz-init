## Overview

- We use Handlebars as our templating language.
- Test templates with `--dev-mode` for rapid development and validation.

## Template Development with Dev Mode

The `--dev-mode` feature provides comprehensive template testing and
validation, dramatically improving the template development workflow.

### Quick Start

Test all configurations for a template:

```bash
fuzz-init --dev-mode --language C
```

This will:

- Test all fuzzer×integration×mode combinations (24 for C template)
- Show build success/failure for each configuration
- Provide detailed error reporting
- Use temporary directories that auto-cleanup

### Development Workflow

1. **Edit your template files** in `src/templates/{language}/`

2. **Test specific configurations** during development:

   ```bash
   # Test only libfuzzer with makefile integration
   fuzz-init --dev-mode --language C --fuzzer libfuzzer --integration make
   ```

3. **Use persistent output** for debugging failed builds:

   ```bash
   fuzz-init --dev-mode --language C --dev-output ./debug-workspace/
   # Project files are preserved in ./debug-workspace/ for inspection
   # CMake logs, build artifacts, and generated code remain available
   ```

4. **Watch mode** for continuous testing (when editing templates):
   ```bash
   fuzz-init --dev-mode --language C --watch src/templates/C/
   # Automatically re-runs tests when template files change
   ```

### Understanding Test Results

Dev mode provides a comprehensive results dashboard:

```
═══════════════════════════════════════
📊 Test Results Summary
═══════════════════════════════════════
Total:      24
✅ Success: 7
❌ Failed:  17

❌ Failed configurations:
  • afl + standalone + full - Build failed: Build script failed with exit code: Some(1)
  • libfuzzer + cmake + minimal - Build failed: CMake configure failed with exit code: Some(1)
```

### Template Testing Best Practices

1. **Start with working configurations**: If you know makefile integration
   works, test that first during development.

2. **Fix one integration at a time**: Use `--integration` flag to focus on
   specific build systems.

3. **Test both modes**: Always verify both `--minimal` and full modes work.

4. **Debug with preserved files**: When using `--dev-output`, project directories are preserved after testing, allowing you to:

   - Examine generated CMakeLists.txt and Makefiles
   - Check CMake configuration logs in `build/CMakeFiles/CMakeConfigureLog.yaml`
   - Inspect build artifacts and error output
   - Manually re-run build commands to debug issues

5. **Check error logs**: Build output is captured for debugging failures.

## MacOS Setup

MacOS clang does not come with libfuzzer support, so you will need to install.
For example,

```bash
brew install llvm
echo 'export PATH="$(brew --prefix llvm)/bin:$PATH"' >> ~/.zshrc
```

Make sure `which clang++` does **not** show macos clang (`/usr/bin/clang++`)

## Style guide

- Use hard line wraps configured at 78 characters. We suggest VSCode extension
  Rewrap.
- Run `cargo fmt` before committing code changes.
- Run `cargo clippy` to check for common mistakes.

## Adding New Templates

1. Create a new directory under `src/templates/{language}/`
2. Add a `template.toml` configuration file (see existing templates)
3. Include all necessary template files with Handlebars placeholders
4. Test your template thoroughly with dev mode:

   ```bash
   fuzz-init --dev-mode --language {language}

   # For debugging specific issues:
   fuzz-init --dev-mode --language {language} --dev-output ./debug/
   ```

5. Debug failures by examining preserved project files in the debug directory
6. Ensure all supported fuzzer and integration combinations work

## Embedding templates

- Templates are _embedded_ on release builds (`cargo build --release`) into the
  binary so that users only need to download `fuzz-init` and everything _just works_.
- Templates are **not** embedded with debug builds (`cargo build`, `cargo run`)
  for DX so that you can change a template and rapidly test it.
