#include "gps.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>

void upload_coordinate(const char* server_url, gps_coordinate_t coord) {
    if (!server_url || !coord.valid) {
        printf("No server configured or invalid coordinate\n");
        return;
    }

    // WARNING: Command injection vulnerability!
    char command[512];
    double lat_degrees = (double)coord.latitude / 1000000.0;
    double lon_degrees = (double)coord.longitude / 1000000.0;

    printf("Uploading coordinate lat=%.6f, lon=%.6f to %s\n",
        lat_degrees, lon_degrees, server_url);

    // Vulnerable: user-controlled server_url gets passed to system()
    snprintf(command, sizeof(command),
        "curl -X POST -H \"Content-Type: application/json\" "
        "-d '{\"latitude\": %.6f, \"longitude\": %.6f}' %s",
        lat_degrees, lon_degrees, server_url);

    system(command); // Command injection point!
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        printf("Usage: %s <gps_data_file> [server_url]\n", argv[0]);
        printf("Example: %s sample.nmea\n", argv[0]);
        printf("Example: %s sample.nmea http://api.example.com/gps\n", argv[0]);
        return 1;
    }

    const char* filename = argv[1];
    const char* server_url = (argc > 2) ? argv[2] : NULL;

    // Open and read GPS data file
    int fd = open(filename, O_RDONLY);
    if (fd == -1) {
        printf("Error: Could not open file '%s'\n", filename);
        return 1;
    }

    char buffer[1024];
    ssize_t bytes_read = read(fd, buffer, sizeof(buffer) - 1);
    close(fd);

    if (bytes_read <= 0) {
        printf("Error: Could not read data from file\n");
        return 1;
    }

    buffer[bytes_read] = '\0';

    // Parse the GPS data
    gps_coordinate_t coord;
    int result = parse_nmea_line(buffer, &coord);

    if (result != 0) {
        printf("Error: Failed to parse GPS data\n");
        return 1;
    }

    printf("Parsed GPS coordinate: lat=%d, lon=%d\n",
        coord.latitude, coord.longitude);

    // Process coordinate (triggers vulnerabilities)
    process_coordinate(coord, 0); // 0 = test all bug types

    // Upload if server URL provided
    if (server_url) {
        upload_coordinate(server_url, coord);
    }

    return 0;
}