{{#unless minimal}}
# CMake configuration for {{project_name}} tests

cmake_minimum_required(VERSION 3.10)

# Test executable
add_executable(test_{{project_name}} test_lib.c)

# Link against the main library
target_link_libraries(test_{{project_name}} {{project_name}}_lib)

# Include directories
target_include_directories(test_{{project_name}} PRIVATE ../include)

# Compiler flags for tests
target_compile_options(test_{{project_name}} PRIVATE -g -O0 -Wall -Wextra -std=c99)

# Custom test target
add_custom_target(run_tests
    COMMAND test_{{project_name}}
    DEPENDS test_{{project_name}}
    COMMENT "Running unit tests"
    VERBATIM
)

# Enable CTest integration
enable_testing()
add_test(NAME lib_tests COMMAND test_{{project_name}})

# Set test properties
set_tests_properties(lib_tests PROPERTIES
    TIMEOUT 30
    PASS_REGULAR_EXPRESSION "All tests passed!"
)
{{else}}
# Minimal mode - no CMake tests generated
# For minimal mode, integrate tests with your existing CMake configuration

message(STATUS "Minimal mode: Tests not included in minimal template")
message(STATUS "For testing guidance, see full mode template or INTEGRATION.md")
{{/unless}}