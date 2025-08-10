#include "mylib.h"
#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <cstdlib>
#include <unistd.h>
#include <fcntl.h>


int main(int argc, char* argv[]) {
    char input[64];
    int fd = STDIN_FILENO;
    ssize_t bytes_read = 0;

    if (argc == 2) {
        std::cout << "Hello fuzz world! Reading from file " << argv[1] << std::endl;
        fd = open(argv[1], O_RDONLY);
        if (fd < 0) {
            std::cerr << "Error: Could not open file '" << argv[1] << "'" << std::endl;
            return -1;
        }
        bytes_read = read(fd, input, sizeof(input) - 1);
        close(fd);
    }
    else {
        std::cout << "Hello fuzz world! Reading from stdin" << std::endl;
        bytes_read = read(fd, input, sizeof(input) - 1);
    }

    if (bytes_read <= 0) {
        std::cerr << "Error: Read no bytes from input" << std::endl;
        return -1;
    }

    input[bytes_read] = '\0';

    // Process input
    try {
        process(input);
    } catch (const std::exception& e) {
        std::cerr << "Exception caught: " << e.what() << std::endl;
        return -1;
    }

    return 0;
}