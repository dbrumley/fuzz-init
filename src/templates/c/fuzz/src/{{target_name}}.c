#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

{{#unless minimal}}
#include "mylib.h"
{{else}}
// TODO: Replace this placeholder with your project's header files
// Example: #include "your_lib.h"
#include <stdio.h>  // For demonstration crash
{{/unless}}

int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
{{#unless minimal}}
    // Example: Process the input data through your library function
    // Note: Ensure data is properly null-terminated if your function expects a string
    if (size > 0) {
        char* null_terminated = (char*)malloc(size + 1);
        if (null_terminated) {
            memcpy(null_terminated, data, size);
            null_terminated[size] = '\0';
            process(null_terminated);
            free(null_terminated);
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
    // Example replacement:
    //   my_parser_result_t result = my_parse_function(data, size);
    //   my_process_data(&result);
    //   my_cleanup(&result);
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
