# CMake configuration for fuzzing {{project_name}} with {{default_fuzzer}}
# This CMakeLists.txt is optimized for your chosen fuzzer and integration

cmake_minimum_required(VERSION 3.16)
project({{project_name}}-fuzz-{{default_fuzzer}})

# Set C standard
set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Set default build type to Debug for fuzzing
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer requires clang
if(NOT CMAKE_C_COMPILER MATCHES "clang")
    message(FATAL_ERROR "libFuzzer requires clang compiler. Please set CMAKE_C_COMPILER to clang.")
endif()

# libFuzzer configuration
set(FUZZER_NAME "libFuzzer")
set(TARGET_NAME "{{target_name}}_libfuzzer")
set(FUZZER_FLAGS "-fsanitize=fuzzer,address")
set(LIBRARY_NAME "{{project_name}}-libfuzzer")
set(FUZZER_DEFINE "DFUZZER_TYPE_LIBFUZZER")

# libFuzzer doesn't need driver (provides its own main)
set(FUZZ_SOURCES "src/{{target_name}}.c")
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# AFL configuration - try to find afl-clang-fast
find_program(AFL_CLANG_FAST afl-clang-fast)
if(AFL_CLANG_FAST)
    set(CMAKE_C_COMPILER ${AFL_CLANG_FAST})
    message(STATUS "Using afl-clang-fast for AFL fuzzing")
else()
    message(FATAL_ERROR "AFL fuzzing requires afl-clang-fast. Please install AFL++ or set CMAKE_C_COMPILER manually.")
endif()

set(FUZZER_NAME "AFL++")
set(TARGET_NAME "{{target_name}}_afl")
set(FUZZER_FLAGS "-fsanitize=address")
set(LIBRARY_NAME "{{project_name}}-afl")
set(FUZZER_DEFINE "DFUZZER_TYPE_AFL")

# AFL needs the universal driver
set(FUZZ_SOURCES "src/{{target_name}}.c" "driver/main.c")
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# HonggFuzz configuration - try to find hfuzz-clang
find_program(HFUZZ_CLANG hfuzz-clang)
if(HFUZZ_CLANG)
    set(CMAKE_C_COMPILER ${HFUZZ_CLANG})
    message(STATUS "Using hfuzz-clang for HonggFuzz fuzzing")
else()
    message(WARNING "hfuzz-clang not found, using regular clang. Install HonggFuzz for optimal performance.")
    if(NOT CMAKE_C_COMPILER MATCHES "clang")
        message(FATAL_ERROR "HonggFuzz requires clang compiler when hfuzz-clang is not available.")
    endif()
endif()

set(FUZZER_NAME "HonggFuzz")
set(TARGET_NAME "{{target_name}}_honggfuzz")
set(FUZZER_FLAGS "-fsanitize=address")
set(LIBRARY_NAME "{{project_name}}-honggfuzz")
set(FUZZER_DEFINE "DFUZZER_TYPE_HONGGFUZZ")

# HonggFuzz needs the universal driver
set(FUZZ_SOURCES "src/{{target_name}}.c" "driver/main.c")
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Standalone fuzzing configuration
if(NOT CMAKE_C_COMPILER MATCHES "clang")
    message(WARNING "clang recommended for better fuzzing support, but other compilers may work")
endif()

set(FUZZER_NAME "Standalone")
set(TARGET_NAME "{{target_name}}_standalone")
set(FUZZER_FLAGS "-fsanitize=address")
set(LIBRARY_NAME "{{project_name}}-fuzz")
set(FUZZER_DEFINE "DFUZZER_TYPE_STANDALONE")

# Standalone needs the universal driver
set(FUZZ_SOURCES "src/{{target_name}}.c" "driver/main.c")
{{/if}}

# Common compiler flags optimized for fuzzing
set(COMMON_FLAGS -g -O2 -Wall -Wextra)

# Include directories - adjust to match your project
include_directories(../include)

# Reference the fuzzing library target from parent directory
{{#if (eq integration "cmake")}}
# For CMake integration, use target dependency instead of file path
set(REQUIRED_LIBRARY_TARGET "${LIBRARY_NAME}")
message(STATUS "Using CMake target dependency: ${REQUIRED_LIBRARY_TARGET}")
{{else}}
# For other integrations, check for library file
set(REQUIRED_LIBRARY "../lib${LIBRARY_NAME}.a")
if(NOT EXISTS ${REQUIRED_LIBRARY})
    message(WARNING 
        "Required fuzzing library not found: ${REQUIRED_LIBRARY}\n"
        "Please run 'make lib-{{#if (eq default_fuzzer "standalone")}}fuzz{{else}}{{default_fuzzer}}{{/if}}' in the parent directory first.\n"
        "This ensures consistent sanitizer instrumentation between your project and fuzz harness.")
else()
    message(STATUS "✓ Found required library: ${REQUIRED_LIBRARY}")
endif()
{{/if}}

# Create the {{default_fuzzer}} fuzzer target
add_executable(${TARGET_NAME} ${FUZZ_SOURCES})

# Set compiler flags and definitions
target_compile_options(${TARGET_NAME} PRIVATE ${COMMON_FLAGS} ${FUZZER_FLAGS})
target_compile_definitions(${TARGET_NAME} PRIVATE ${FUZZER_DEFINE})

# Link against the fuzzing library
{{#if (eq integration "cmake")}}
target_link_libraries(${TARGET_NAME} PRIVATE ${REQUIRED_LIBRARY_TARGET})
{{else}}
target_link_libraries(${TARGET_NAME} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/${REQUIRED_LIBRARY})
{{/if}}

# Set linker flags for sanitizers
target_link_options(${TARGET_NAME} PRIVATE ${FUZZER_FLAGS})

# Custom target for running the fuzzer
{{#if (eq default_fuzzer "standalone")}}
add_custom_target(run-fuzzer
    COMMAND ./${TARGET_NAME} ../testsuite/
    DEPENDS ${TARGET_NAME}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "Running standalone fuzzer on test suite"
)
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# Find AFL runner
find_program(AFL_FUZZ afl-fuzz)
if(AFL_FUZZ)
    add_custom_target(run-fuzzer
        COMMAND ${CMAKE_COMMAND} -E make_directory findings
        COMMAND ${AFL_FUZZ} -i ../testsuite -o findings -- ./${TARGET_NAME}
        DEPENDS ${TARGET_NAME}
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
        COMMENT "Running AFL fuzzer"
    )
else()
    message(STATUS "afl-fuzz not found, run-fuzzer target will not be available")
endif()
{{/if}}

{{#if (eq default_fuzzer "libfuzzer")}}
add_custom_target(run-fuzzer
    COMMAND ${CMAKE_COMMAND} -E make_directory corpus
    COMMAND ./${TARGET_NAME} corpus/ -dict=../dictionaries/{{target_name}}.dict -max_total_time=300
    DEPENDS ${TARGET_NAME}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "Running libFuzzer"
)
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# Find HonggFuzz runner
find_program(HONGGFUZZ honggfuzz)
if(HONGGFUZZ)
    add_custom_target(run-fuzzer
        COMMAND ${CMAKE_COMMAND} -E make_directory corpus
        COMMAND ${HONGGFUZZ} -i ../testsuite/ -W corpus/ -- ./${TARGET_NAME}
        DEPENDS ${TARGET_NAME}
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
        COMMENT "Running HonggFuzz"
    )
else()
    message(STATUS "honggfuzz not found, run-fuzzer target will not be available")
endif()
{{/if}}

# Test target - builds and runs basic smoke test
add_custom_target(test-fuzzer
{{#if (eq default_fuzzer "libfuzzer")}}
    COMMAND ./${TARGET_NAME} ../testsuite/ -runs=10
{{else}}
    COMMAND echo "FUZZ" | ./${TARGET_NAME}
{{/if}}
    DEPENDS ${TARGET_NAME}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "Running smoke test with ${FUZZER_NAME}"
)

# Copy necessary files to build directory
file(GLOB TESTSUITE_FILES "../testsuite/*")
foreach(FILE ${TESTSUITE_FILES})
    get_filename_component(FILENAME ${FILE} NAME)
    configure_file(${FILE} ${CMAKE_CURRENT_BINARY_DIR}/testsuite/${FILENAME} COPYONLY)
endforeach()

if(EXISTS "../dictionaries/{{target_name}}.dict")
    configure_file("../dictionaries/{{target_name}}.dict" 
                   "${CMAKE_CURRENT_BINARY_DIR}/dictionaries/{{target_name}}.dict" COPYONLY)
endif()

# Print build information
message(STATUS "")
message(STATUS "🎯 Fuzzing configuration for {{project_name}}:")
message(STATUS "  Fuzzer:       ${FUZZER_NAME}")
message(STATUS "  Target:       ${TARGET_NAME}")
message(STATUS "  Compiler:     ${CMAKE_C_COMPILER}")
message(STATUS "  Library:      ${REQUIRED_LIBRARY}")
message(STATUS "  Integration:  {{integration}}")
message(STATUS "")
message(STATUS "📋 Available targets:")
message(STATUS "  ${TARGET_NAME}  - Build the fuzzer")
message(STATUS "  run-fuzzer     - Build and run the fuzzer")
message(STATUS "  test-fuzzer    - Build and run smoke test")
message(STATUS "")
message(STATUS "🚀 Usage:")
message(STATUS "  cmake --build . --target ${TARGET_NAME}")
{{#if (eq default_fuzzer "libfuzzer")}}
message(STATUS "  ./${TARGET_NAME} testsuite/ -dict=dictionaries/{{target_name}}.dict")
{{/if}}
{{#if (eq default_fuzzer "afl")}}
message(STATUS "  afl-fuzz -i testsuite -o findings -- ./${TARGET_NAME}")
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
message(STATUS "  honggfuzz -i testsuite/ -W corpus/ -- ./${TARGET_NAME}")
{{/if}}
{{#if (eq default_fuzzer "standalone")}}
message(STATUS "  ./${TARGET_NAME} testsuite/")
{{/if}}
message(STATUS "")