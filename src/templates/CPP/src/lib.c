#include "lib.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <assert.h>

// Helper functions for vulnerable behaviors
void divide_by_zero_bug(int x, int y) {
    volatile int res = 0;

    if (x == 1) {
        res = x / y; // Divide by zero.    Often a silent, serious vuln.
    }
}

void integer_overflow_bug(int x, int y) {
    volatile int res = 0;
    if (x == 2 && y == -79927771) {
        res = x - y + 2067556990; // Integer overflow.  Often a silent, serious vuln. 
    }
}

void oob_read_bug(int x, int y) {
    volatile char oob_data;
    char buffer[8];
    strcpy(buffer, "AAAAAA"); // Completely safe
    if (x == 3 && y == -79927771) {
        oob_data = buffer[x - y]; // Out of bounds read
    }
}

void oob_write_bug(int x, int y) {
    char buffer[8];
    strcpy(buffer, "AAAAAA"); // Completely safe
    if (x == 4 && y == -79927771) {
        buffer[x - y] = 'X'; // Out of bounds write
    }
}

void double_free_bug(int x, int y) {
    char* buf = malloc(x > 0 ? x : 16);
    free(buf);
    if (x == 5 && y == -79927771) {
        free(buf); // Double free
    }
}

void stack_exhaustion_bug(int x, int y) {
    char stack_buffer[0x1000];
    // Prevent compiler optimization
    memset(stack_buffer, 0, sizeof(stack_buffer));
    if (x == 6 && y == -79927771) {
        stack_exhaustion_bug(x, y); // Infinite recursion
    }
}

void assert_bug(int x, int y) {
    if (x == 7 && y == 7) {
        assert(0); // Trigger assertion failure
    }
}

int process(char* input) {
    char* str = NULL;
    char* fields[2];
    int field_count = 0;

    str = input;

    fields[field_count++] = str;
    for (char* ptr = str; *ptr != '\0' && field_count < 15; ptr++) {
        if (*ptr == ',') {
            *ptr = '\0'; // Terminate the current field
            fields[field_count++] = ptr + 1; // Start of the next field
        }
    }
    if (field_count == 2) {

        int x = atoi(fields[0]);
        int y = atoi(fields[1]);

        divide_by_zero_bug(x, y);
        integer_overflow_bug(x, y);
        oob_read_bug(x, y);
        oob_write_bug(x, y);
        double_free_bug(x, y);
        stack_exhaustion_bug(x, y);
        assert_bug(x, y);
    }
    else {
        printf("Error: Invalid input format. Expected two comma-separated integers.\n");
        return -1;
    }
    return 0;
}