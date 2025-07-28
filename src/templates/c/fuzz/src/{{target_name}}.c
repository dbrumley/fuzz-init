#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

{{#if (eq minimal false)}}
#include "lib.h"
{{else}}
// TODO: Add your project's header files here
// Example: #include "your_lib.h"
{{/if}}

int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size) {
{{#if (eq minimal false)}}
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
    // TODO: Add your fuzzing logic here
    // This function is called by the fuzzer with test data
    // 
    // Example:
    // - Parse the input data according to your format
    // - Call your library functions with the parsed data
    // - Handle any necessary cleanup
    //
    // Tips:
    // - The fuzzer will call this function repeatedly with different inputs
    // - Avoid using global state that persists between calls
    // - Consider adding bounds checking for size
    // - Remember to free any allocated memory
    
    // Placeholder to prevent unused parameter warnings
    (void)data;
    (void)size;
{{/if}}
    return 0;
}
