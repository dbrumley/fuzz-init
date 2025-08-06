{{#unless minimal}}
# CMake configuration for {{project_name}} tests

cmake_minimum_required(VERSION 3.10)

# Create safe target names (replace slashes with underscores)
string(REPLACE "/" "_" PROJECT_TARGET_NAME "{{project_name}}")

# Test executable
add_executable(test_${PROJECT_TARGET_NAME} test_lib.cpp)

# Link against the main library  
target_link_libraries(test_${PROJECT_TARGET_NAME} ${PROJECT_TARGET_NAME})

# Include directories
target_include_directories(test_${PROJECT_TARGET_NAME} PRIVATE ../include)

# Compiler flags for tests
target_compile_options(test_${PROJECT_TARGET_NAME} PRIVATE -g -O0 -Wall -Wextra -std=c++11)

# Custom test target
add_custom_target(run_tests
    COMMAND test_${PROJECT_TARGET_NAME}
    DEPENDS test_${PROJECT_TARGET_NAME}
    COMMENT "Running unit tests"
    VERBATIM
)

# Enable CTest integration
enable_testing()
add_test(NAME lib_tests COMMAND test_${PROJECT_TARGET_NAME})

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