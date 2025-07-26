#ifndef GPS_PARSER_H
#define GPS_PARSER_H

#include <stdint.h>
#include <stddef.h>

typedef struct {
    int latitude;
    int longitude;
    int valid;
} gps_coordinate_t;

// Parse NMEA-style GPS line into coordinate structure
// Returns 0 on success, -1 on parse error
// WARNING: Contains vulnerabilities for fuzzing demonstration!
int parse_nmea_line(const char* line, gps_coordinate_t* coord);

// Process GPS coordinate with various bug triggers
// bug_trigger: 0 = all bugs, 1-6 = specific bug types
// WARNING: Contains intentional vulnerabilities!
void process_coordinate(gps_coordinate_t coord, int bug_trigger);

// Upload coordinate to server (simulated)
// May contain command injection vulnerabilities
void upload_coordinate(const char* server_url, gps_coordinate_t coord);

#endif // GPS_PARSER_H