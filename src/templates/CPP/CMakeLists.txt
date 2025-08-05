cmake_minimum_required(VERSION 3.10)
project({{project_name}} C)

set(CMAKE_C_STANDARD 99)

# Compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -g")
set(CMAKE_C_FLAGS_DEBUG "-O0 -fsanitize=address")
set(CMAKE_C_FLAGS_RELEASE "-O2")

# AddressSanitizer for debug builds
if(CMAKE_BUILD_TYPE STREQUAL "Debug" OR NOT CMAKE_BUILD_TYPE)
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fsanitize=address")
endif()

# Source files
set(LIB_SOURCES src/lib.c)

# Create main library
add_library({{project_name}}_lib STATIC ${LIB_SOURCES})
target_include_directories({{project_name}}_lib PUBLIC include)
target_compile_options({{project_name}}_lib PRIVATE -g -O0)

# Default fuzzer library based on template selection
{{#if (eq default_fuzzer "libfuzzer")}}
set(DEFAULT_FUZZER_LIB {{project_name}}-libfuzzer)
{{/if}}
{{#if (eq default_fuzzer "afl")}}
set(DEFAULT_FUZZER_LIB {{project_name}}-afl)
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
set(DEFAULT_FUZZER_LIB {{project_name}}-honggfuzz)
{{/if}}
{{#if (eq default_fuzzer "standalone")}}
set(DEFAULT_FUZZER_LIB {{project_name}}-fuzz)
{{/if}}

# Create main executable
add_executable({{target_name}} src/main.c)
target_link_libraries({{target_name}} {{project_name}}_lib)

# Create test data directory target
add_custom_target(test_data
    COMMAND ${CMAKE_COMMAND} -E copy_directory ${CMAKE_SOURCE_DIR}/test_data ${CMAKE_BINARY_DIR}/test_data
    COMMENT "Copying test data to build directory"
)

{{#unless minimal}}
# Add test subdirectory (unit tests)
add_subdirectory(test)

# Custom target for integration testing with sample data
add_custom_target(integration_test
    COMMAND $<TARGET_FILE:{{target_name}}> ${CMAKE_BINARY_DIR}/test_data/valid.nmea
    COMMAND echo "=== Out of bound write (OOB Write) ==="
    COMMAND $<TARGET_FILE:{{target_name}}> ${CMAKE_BINARY_DIR}/test_data/oob_read.nmea
    DEPENDS {{target_name}} test_data
    COMMENT "Running integration tests with sample data"
)

# Override default test target to run unit tests
add_custom_target(test
    COMMAND ${CMAKE_COMMAND} --build . --target run_tests
    DEPENDS test_{{project_name}}
    COMMENT "Running unit tests"
)
{{else}}
# Minimal mode - no unit tests, only basic integration test
add_custom_target(test
    COMMAND echo "Minimal mode: Unit tests not included"
    COMMAND echo "For testing guidance, see full mode template or fuzz/INTEGRATION.md"
    COMMENT "Testing info for minimal mode"
)
{{/unless}}

# Fuzz target - builds the default fuzzer
add_custom_target(fuzz
    COMMAND ${CMAKE_COMMAND} --build fuzz --target {{target_name}}_{{default_fuzzer}}
    DEPENDS ${DEFAULT_FUZZER_LIB}
    COMMENT "Building {{default_fuzzer}} fuzzer"
)

# Additional fuzzer-specific library targets
# These match the naming expected by fuzz/CMakeLists.txt
add_library({{project_name}}-afl STATIC ${LIB_SOURCES})
target_include_directories({{project_name}}-afl PUBLIC include)
target_compile_options({{project_name}}-afl PRIVATE -g -O1 -fsanitize=address)

add_library({{project_name}}-libfuzzer STATIC ${LIB_SOURCES})
target_include_directories({{project_name}}-libfuzzer PUBLIC include)
target_compile_options({{project_name}}-libfuzzer PRIVATE -g -O1 -fsanitize=address,fuzzer-no-link)

add_library({{project_name}}-honggfuzz STATIC ${LIB_SOURCES})
target_include_directories({{project_name}}-honggfuzz PUBLIC include)  
target_compile_options({{project_name}}-honggfuzz PRIVATE -g -O1 -fsanitize=address)

add_library({{project_name}}-fuzz STATIC ${LIB_SOURCES})
target_include_directories({{project_name}}-fuzz PUBLIC include)
target_compile_options({{project_name}}-fuzz PRIVATE -g -O1 -fsanitize=address)

# Subdirectory for fuzz targets
add_subdirectory(fuzz)

# Install targets
install(TARGETS {{target_name}} DESTINATION bin)
