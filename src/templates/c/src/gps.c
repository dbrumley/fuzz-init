#include "gps.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Helper functions for vulnerable behaviors
static void divide_by_zero_bug(int lat, int lon) {
    volatile int res = 0;
    if (lat == 1)
        res = lat / lon; // Divide by zero when lon = 0
}

static void integer_overflow_bug(int lat, int lon) {
    volatile int res = 0;
    if (lat == 2 && lon == -79927771) {
        res = lat - lon + 2067556990; // Integer overflow
    }
}

static void oob_read_bug(int lat, int lon) {
    volatile char oob_data;
    char buffer[8];
    strcpy(buffer, "AAAAAA");
    if (lat == 3 && lon == -79927771) {
        oob_data = buffer[lat - lon]; // Out of bounds read
    }
}

static void oob_write_bug(int lat, int lon) {
    char buffer[8];
    strcpy(buffer, "AAAAAA");
    if (lat == 4 && lon == -79927771) {
        buffer[lat - lon] = 'X'; // Out of bounds write
    }
}

static void double_free_bug(int lat, int lon) {
    char* buf = malloc(lat > 0 ? lat : 16);
    free(buf);
    if (lat == 5 && lon == -79927771) {
        free(buf); // Double free
    }
}

static void stack_exhaustion_bug(int lat, int lon) {
    char stack_buffer[0x1000];
    // Prevent compiler optimization
    memset(stack_buffer, 0, sizeof(stack_buffer));
    if (lat == 6 && lon == -79927771) {
        stack_exhaustion_bug(lat, lon); // Infinite recursion
    }
}

int parse_nmea_line(const char* line, gps_coordinate_t* coord) {
    if (!line || !coord) {
        return -1;
    }

    // Initialize coordinate
    coord->latitude = 0;
    coord->longitude = 0;
    coord->valid = 0;

    // Simple CSV-style parsing (simulating NMEA format)
    // Expected format: "TYPE,TIME,LAT,LAT_DIR,LON,LON_DIR,..."
    char* line_copy = strdup(line);
    if (!line_copy) {
        return -1;
    }

    char* fields[15];
    int field_count = 0;
    char* token = line_copy;

    // Split by commas
    fields[field_count++] = token;
    for (char* ptr = token; *ptr != '\0' && field_count < 15; ptr++) {
        if (*ptr == ',') {
            *ptr = '\0';
            fields[field_count++] = ptr + 1;
        }
    }

    // Need at least 6 fields for basic GPS data
    if (field_count < 6) {
        free(line_copy);
        return -1;
    }

    // Parse latitude (field 2) and longitude (field 4)
    int lat = atoi(fields[2]);
    int lon = atoi(fields[4]);

    // Apply direction indicators (fields 3 and 5)
    if (field_count > 3 && fields[3][0] == 'S') lat = -lat;
    if (field_count > 5 && fields[5][0] == 'W') lon = -lon;

    coord->latitude = lat;
    coord->longitude = lon;
    coord->valid = 1;

    free(line_copy);
    return 0;
}

void process_coordinate(gps_coordinate_t coord, int bug_trigger) {
    if (!coord.valid) {
        return;
    }

    int lat = coord.latitude;
    int lon = coord.longitude;

    // Trigger various vulnerabilities based on bug_trigger
    switch (bug_trigger) {
    case 0: // All bugs
        divide_by_zero_bug(lat, lon);
        integer_overflow_bug(lat, lon);
        oob_read_bug(lat, lon);
        oob_write_bug(lat, lon);
        double_free_bug(lat, lon);
        stack_exhaustion_bug(lat, lon);
        break;
    case 1: integer_overflow_bug(lat, lon); break;
    case 2: divide_by_zero_bug(lat, lon); break;
    case 3: oob_read_bug(lat, lon); break;
    case 4: oob_write_bug(lat, lon); break;
    case 5: double_free_bug(lat, lon); break;
    case 6: stack_exhaustion_bug(lat, lon); break;
    default:
        printf("Processing coordinate: lat=%d, lon=%d\n", lat, lon);
        break;
    }
}
