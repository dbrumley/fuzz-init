#include <cstdint>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <string>
#include <memory>


{{#unless minimal}}
#include <vector>
#include "mylib.h"
{{else}}
// TODO: Include your own header files
// Example:
// #include "include/parse.hpp" // include files from ../include/parse.hpp"

{{/unless}}

// Link with the universal C driver (driver/main.c) used by AFL, HonggFuzz, and
// standalone modes. Libfuzzer doesn't need this because it provides it's
// own main() function.  
extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
    {{ #unless minimal }}
    // Example: Process the input data through your library function
    // Note: Ensure data is properly null-terminated if your function expects a string
    // Reject empty inputs
    if (size == 0) {
        return 0;
    }

    // Allocate a null-terminated buffer with space for the data + '\0'
    std::vector<char> input_buffer(size + 1, 0);  // zero-initialized
    std::memcpy(input_buffer.data(), data, size); // copy raw data

    // Call the C function safely
    try {
        return process(input_buffer.data());
    } catch (...) {
        // Prevent exceptions from escaping; C code shouldn't throw,
        // but just in case anything goes wrong.
    }

    return 0;
    {{else}}
    // ========================================================================
    // DEMO: Using the demonstration library
    // ========================================================================
    //
    // To integrate YOUR library:
    // 1. Replace the demo function below with a call to the function you want
    //    to fuzz test
    // 3. Rebuild your library with proper fuzz instrumentation
    // ========================================================================

    // Simple demonstration: crash if input contains "bug"
    if (size >= 3) {
        for (size_t i = 0; i <= size - 3; i++) {
            if (data[i] == 'b' && data[i + 1] == 'u' && data[i + 2] == 'g') {
                printf("Found the bug! Crashing as demonstration...\n");
                // This will be caught by AddressSanitizer or cause a crash
                int* crash = NULL;
                *crash = 42;  // Intentional crash for demo
            }
        }
    }

    // TODO: Remove the above demonstration code and add your logic here
    return 0;

    {{/unless}}
    
}
