# libadm Integration Workflow Documentation

This document outlines the correct workflow for integrating fuzz-init with existing projects, using libadm as a real-world example.

## Current Status

### ✅ Completed Tasks
1. **Fixed filesystem templating bug** - {{target_name}}.c files now created correctly in --minimal mode
2. **Created C++ template** - Full-featured C++ template based on C template structure  
3. **Fixed dictionary filename templating** - Dictionary files now properly renamed from {{target_name}}.dict
4. **Enhanced fuzz harness** - Created realistic libadm XML parsing fuzz harness

### 🔄 Current Integration Challenge

The current challenge is properly integrating the generated fuzz harness with libadm's existing CMake build system.

#### Problem
The generated CMakeLists.txt expects a library target named "libadm-fuzz-libfuzzer" but libadm provides a target named "adm".

#### Root Cause
The C++ template CMakeLists.txt is designed for standalone projects that build their own library, not for integration with existing CMake projects.

## Working libadm Fuzz Harness Code

### Generated fuzz/src/libadm-fuzz.cpp
```cpp
#include <cstdint>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <string>
#include <memory>
#include <sstream>

// libadm headers
#include "adm/parse.hpp"
#include "adm/document.hpp"

int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
    // Fuzz libadm XML parsing
    try {
        // Convert input to string for XML parsing
        std::string xml_input(reinterpret_cast<const char*>(data), size);
        
        // Create string stream for parsing
        std::istringstream xml_stream(xml_input);
        
        // Parse the XML document using libadm
        auto document = adm::parseXml(xml_stream);
        
        // If parsing succeeds, perform some operations on the document
        if (document) {
            // Try to access document elements to trigger potential bugs
            document->getElements<adm::AudioProgramme>();
            document->getElements<adm::AudioContent>();
            document->getElements<adm::AudioObject>();
            document->getElements<adm::AudioPackFormat>();
            document->getElements<adm::AudioChannelFormat>();
            document->getElements<adm::AudioStreamFormat>();
            document->getElements<adm::AudioTrackFormat>();
            document->getElements<adm::AudioTrackUid>();
        }
        
    } catch (const std::exception& e) {
        // Expected for malformed XML - just return
        // The fuzzer is looking for crashes, not exceptions
    } catch (...) {
        // Catch any other unexpected exceptions
    }
    
    return 0;
}
```

## Required Improvements

### 1. CMake Integration Template Enhancement
The C++ template needs an improved CMakeLists.txt that can:
- Detect existing CMake targets in parent directory
- Use existing target names instead of generating new ones
- Provide fallback behavior for standalone usage

### 2. Project Detection Logic
fuzz-init should detect:
- Existing CMakeLists.txt files
- Available library targets  
- Project language from source files
- Build system type (CMake, Make, etc.)

### 3. Manual Integration Workflow
For now, the manual workflow is:

1. Generate minimal fuzz directory:
   ```bash
   fuzz-init libadm-fuzz --language CPP --fuzzer libfuzzer --integration cmake --minimal
   ```

2. Manually edit fuzz/CMakeLists.txt to use correct target:
   ```cmake
   set(REQUIRED_LIBRARY_TARGET "adm")  # Not "libadm-fuzz-libfuzzer"
   ```

3. Build from parent directory with subdirectory:
   ```cmake
   # Add to parent CMakeLists.txt
   add_subdirectory(libadm-fuzz/fuzz)
   ```

## Next Development Priorities

1. **Enhance CMake integration template** to detect and use existing targets
2. **Add project detection logic** to automatically identify build systems and targets
3. **Create integration guides** for common CMake patterns
4. **Test workflow** with multiple real-world projects

## Success Criteria

✅ Generate working fuzz harness code
✅ Create proper directory structure  
✅ Template filename correctly
🔄 Build successfully with parent project (requires CMake template fixes)
❌ Run fuzzer and find bugs (blocked by build issues)

The core templating system is working correctly - the remaining work is improving the CMake integration to handle existing projects better.