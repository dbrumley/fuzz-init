{{#unless minimal}}
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include "lib.h"

// Simple test framework macros
#define TEST_PASSED 0
#define TEST_FAILED 1

static int tests_run = 0;
static int tests_passed = 0;

#define RUN_TEST(test) do { \
    printf("Running %s... ", #test); \
    if (test() == TEST_PASSED) { \
        printf("PASSED\n"); \
        tests_passed++; \
    } else { \
        printf("FAILED\n"); \
    } \
    tests_run++; \
} while(0)

// Test the main process function with valid inputs
int test_process_valid_input() {
    char input1[] = "0,1";  // Safe values that won't trigger bugs
    char input2[] = "10,20"; // Safe values
    char input3[] = "-5,15"; // Safe values
    
    // These should return 0 (success) without crashing
    if (process(input1) != 0) return TEST_FAILED;
    if (process(input2) != 0) return TEST_FAILED;
    if (process(input3) != 0) return TEST_FAILED;
    
    return TEST_PASSED;
}

// Test the process function with invalid input formats
int test_process_invalid_input() {
    char input1[] = "not_a_number";
    char input2[] = "1";  // Only one field
    char input3[] = "1,2,3";  // Too many fields
    char input4[] = "";   // Empty input
    
    // These should return -1 (error) for invalid format
    if (process(input1) != -1) return TEST_FAILED;
    if (process(input2) != -1) return TEST_FAILED;
    if (process(input3) != -1) return TEST_FAILED;
    if (process(input4) != -1) return TEST_FAILED;
    
    return TEST_PASSED;
}

// Test individual bug functions with safe parameters
// Note: We test with safe values to ensure the functions exist and run
// without triggering the actual bugs (which would crash the test)
int test_bug_functions_safe() {
    // Call each bug function with safe parameters that won't trigger bugs
    divide_by_zero_bug(0, 1);  // x != 1, so no division by zero
    integer_overflow_bug(0, 1);  // x != 2, so no overflow
    oob_read_bug(0, 1);  // x != 3, so no out-of-bounds read
    oob_write_bug(0, 1);  // x != 4, so no out-of-bounds write
    double_free_bug(0, 1);  // x != 5, so no double free
    // Note: We don't test stack_exhaustion_bug as it's recursive
    // and assert_bug as it would terminate the test program
    
    return TEST_PASSED;
}

// Test input parsing edge cases
int test_input_parsing() {
    char input1[] = "  5  ,  10  ";  // Spaces should be handled by atoi
    char input2[] = "0,0";           // Zero values
    char input3[] = "-1,-1";         // Negative values
    
    // These should parse correctly and return 0
    if (process(input1) != 0) return TEST_FAILED;
    if (process(input2) != 0) return TEST_FAILED;
    if (process(input3) != 0) return TEST_FAILED;
    
    return TEST_PASSED;
}

// Test memory handling (basic smoke test)
int test_memory_handling() {
    // Test with various input sizes
    char small_input[] = "1,2";
    char medium_input[] = "12345,67890";
    
    if (process(small_input) != 0) return TEST_FAILED;
    if (process(medium_input) != 0) return TEST_FAILED;
    
    return TEST_PASSED;
}

// Test string modification behavior
int test_string_modification() {
    // The process function modifies the input string (replaces ',' with '\0')
    char input[] = "8,9";
    char original[] = "8,9";
    
    int result = process(input);
    
    // Should succeed
    if (result != 0) return TEST_FAILED;
    
    // Input should be modified (comma replaced with null terminator)
    if (strcmp(input, original) == 0) return TEST_FAILED;  // Should be different
    if (strcmp(input, "8") != 0) return TEST_FAILED;       // First part should be "8"
    
    return TEST_PASSED;
}

int main() {
    printf("=== {{project_name}} Library Test Suite ===\n\n");
    
    // Run all tests
    RUN_TEST(test_process_valid_input);
    RUN_TEST(test_process_invalid_input);
    RUN_TEST(test_bug_functions_safe);
    RUN_TEST(test_input_parsing);
    RUN_TEST(test_memory_handling);
    RUN_TEST(test_string_modification);
    
    // Print summary
    printf("\n=== Test Results ===\n");
    printf("Tests run: %d\n", tests_run);
    printf("Tests passed: %d\n", tests_passed);
    printf("Tests failed: %d\n", tests_run - tests_passed);
    
    if (tests_passed == tests_run) {
        printf("✅ All tests passed!\n");
        return 0;
    } else {
        printf("❌ Some tests failed!\n");
        return 1;
    }
}
{{else}}
// Minimal mode - no tests generated
// This file is only created in full mode to demonstrate testing best practices
// For minimal mode, integrate your own tests with your existing project structure
{{/unless}}