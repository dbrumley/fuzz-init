#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <gps.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Null-terminate the input data
    char* input = (char*)malloc(size + 1);
    if (!input) return 0;
    
    memcpy(input, data, size);
    input[size] = '\0';
    
    // Parse GPS data
    gps_coordinate_t coord;
    int result = parse_nmea_line(input, &coord);
    
    // If parsing succeeded, process the coordinate
    if (result == 0 && coord.valid) {
        // Test all bug triggers (0 = all bugs)
        process_coordinate(coord, 0);
    }
    
    free(input);
    return 0;
}
