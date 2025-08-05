#include <cstdint>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <string>
#include <memory>

{{#unless minimal}}
#include "lib.h"
{{else}}
// TODO: Replace this placeholder with your project's header files
// Example for libadm:
// #include "adm/parse.hpp"
// #include "adm/write.hpp"
#include <iostream>  // For demonstration crash
{{/unless}}

// C++ name mangling prevention - required when using C driver
// The 'extern "C"' linkage is essential for C++ fuzz harnesses to work with
// the universal C driver (driver/main.c) used by AFL, HonggFuzz, and standalone modes.
// LibFuzzer doesn't need this because it provides its own main() function.
extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
{{#unless minimal}}
    // Example: Process the input data through your library function
    // Note: Ensure data is properly null-terminated if your function expects a string
    if (size > 0) {
        std::string input(reinterpret_cast<const char*>(data), size);
        try {
            process(input);
        } catch (const std::exception& e) {
            // Exceptions are handled by libFuzzer - they're treated as interesting inputs
            // but not crashes (unless AddressSanitizer detects memory errors)
        }
    }
{{else}}
    // ========================================================================
    // TODO: REPLACE THIS PLACEHOLDER WITH YOUR ACTUAL FUZZING CODE
    // ========================================================================
    //
    // This is a simple demonstration that will be found by any fuzzer.
    // Replace this entire block with calls to your actual library functions.
    //
    // Example for libadm (XML parsing library):
    //   try {
    //       std::string xml_input(reinterpret_cast<const char*>(data), size);
    //       auto document = adm::parseXml(xml_input);
    //       // Process the parsed document...
    //   } catch (const std::exception& e) {
    //       // Expected for malformed XML - just return
    //   }
    //
    // Example for other libraries:
    //   MyClass obj;
    //   obj.process_data(data, size);
    //
    // The fuzzer will call this function repeatedly with different inputs
    // to find crashes, memory errors, and other bugs in your code.
    // ========================================================================
    
    // Simple demonstration: crash if input contains "bug"
    if (size >= 3) {
        for (size_t i = 0; i <= size - 3; i++) {
            if (data[i] == 'b' && data[i+1] == 'u' && data[i+2] == 'g') {
                printf("Found the bug! Crashing as demonstration...\n");
                // This will be caught by AddressSanitizer or cause a crash
                int* crash = NULL;
                *crash = 42;  // Intentional crash for demo
            }
        }
    }
    
    // TODO: Remove the above demonstration code and add your logic here
{{/unless}}
    return 0;
}
