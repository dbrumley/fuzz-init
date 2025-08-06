# 🎯 {{project_name}} Fuzzing Setup Complete!

{{#if minimal}}
## Quick Start (Minimal Mode)
Your fuzzing harness has been added to `fuzz/` directory.

### 1. Configure and Build
```bash
cd fuzz
./configure.sh  # Auto-configures CMake and builds
```

### 2. Customize Your Harness
Edit `src/{{target_name}}.cpp` to call your library functions.
{{#if detected_xml_lib}}
**Detected XML Library**: See examples in harness for XML parsing usage.
{{/if}}
{{#if detected_library_target}}
**Auto-detected Library**: `{{detected_library_target}}` - already configured for linking.
{{/if}}

### 3. Handle Sanitizers (Important!)
Your main project needs AddressSanitizer for effective fuzzing:
```bash
cd ..  # Back to project root
cmake -S . -B build-fuzz -DCMAKE_CXX_FLAGS="-fsanitize=address -g"
cd fuzz && cmake -S . -B build -DPARENT_BUILD_DIR=../build-fuzz
```

See `INTEGRATION.md` for complete details.
{{else}}
## Full Tutorial Mode
Complete project with examples and documentation created.

### Quick Start
```bash
cd {{project_name}}
make && make test  # Build library and run tests  
make fuzz-{{default_fuzzer}}  # Build fuzzer
./fuzz/{{target_name}}-{{default_fuzzer}} fuzz/testsuite/
```
{{/if}}

## Next Steps
1. 📖 Read `{{#if minimal}}fuzz/{{/if}}INTEGRATION.md` - Complete integration guide
2. ✏️ Modify `{{#if minimal}}fuzz/{{/if}}src/{{target_name}}.cpp` - Customize your harness  
3. 🏗️ Run `{{#if minimal}}cd fuzz && {{/if}}./configure.sh` - Auto-setup build
4. 🐛 Start fuzzing with `./{{target_name}}_{{default_fuzzer}} {{#if minimal}}../{{/if}}testsuite/`

## 📚 Documentation
{{#if minimal}}
- `fuzz/INTEGRATION.md` - Integration with existing projects
- `fuzz/README.md` - Quick fuzzing reference
- `fuzz/src/{{target_name}}.cpp` - Your harness (customize this!)
{{else}}
- `TUTORIAL.md` - Complete fuzzing tutorial and examples
- `fuzz/INTEGRATION.md` - Integration guide for existing projects
- `fuzz/README.md` - Quick reference for fuzzing commands
{{/if}}

{{#if (eq integration "cmake")}}
## CMake Integration Notes
{{#if detected_library_target}}
✓ Auto-detected library target: `{{detected_library_target}}`
{{/if}}
{{#if cmake_version}}
✓ Build system: CMake {{cmake_version}}
{{/if}}
{{#if sanitizer_mismatch_risk}}
⚠️ Sanitizer mismatch possible - see {{#if minimal}}fuzz/{{/if}}INTEGRATION.md for rebuild steps
{{/if}}
{{/if}}

{{#if (eq integration "makefile")}}
## Makefile Integration Notes
{{#if minimal}}
✓ Self-contained Makefile in `fuzz/` directory
✓ Links against parent library: `../lib{{project_name}}.a`
ℹ️ Run `make lib-{{default_fuzzer}}` in parent directory first
{{else}}
✓ Integrated Makefile targets for all fuzzer types
✓ Comprehensive build system with unit tests
{{/if}}
{{/if}}

---
**Happy Fuzzing! 🐛** Run `{{#if minimal}}cd fuzz && {{/if}}./configure.sh` to get started.