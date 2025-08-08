#ifndef LIB_H
#define LIB_H
#include <string>

/**
 * The core implementation of the executable
 *
 * This file describes library functions used by the core executable. This kind
 * of separation makes code easy to test because the logic is nicely separated
 * from the command-line logic implemented in the main function.
 */

 // For C++ projects, we don't need extern "C" blocks unless we're specifically
 // designing for C compatibility. Since this is a C++ template, we'll use
 // regular C++ declarations.

int process(char* input);
void divide_by_zero_bug(int x, int y);
void integer_overflow_bug(int x, int y);
void oob_read_bug(int x, int y);
void oob_write_bug(int x, int y);
void double_free_bug(int x, int y);
void stack_exhaustion_bug(int x, int y);

#endif 