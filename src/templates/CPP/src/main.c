#include "lib.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>


int main(int argc, char* argv[]) {
    char input[64];
    int fd = STDIN_FILENO;
    ssize_t bytes_read = 0;

    if (argc == 2) {
        printf("Hello fuzz world! Reading from file %s\n", argv[1]);
        fd = open(argv[1], O_RDONLY);
        if (fd < 0) {
            printf("Error: Could not open file '%s'\n", argv[1]);
            return -1;
        }
        bytes_read = read(fd, input, sizeof(input) - 1);
        close(fd);
    }
    else {
        printf("Hello fuzz world! Reading from stdin\n");
        bytes_read = read(fd, input, sizeof(input) - 1);
    }


    if (bytes_read <= 0) {
        printf("Error: Read no bytes from input\n");
        return -1;
    }

    input[bytes_read] = '\0';

    // Process input
    process(input);

    return 0;
}