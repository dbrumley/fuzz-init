# CMake configuration for fuzzing {{project_name}}
# This CMakeLists.txt integrates with your existing CMake build system

cmake_minimum_required(VERSION 3.16)
project({{project_name}}-fuzz)

# Set C standard
set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Set default build type to Debug for fuzzing
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

# Find clang compiler (recommended for fuzzing)
if(NOT CMAKE_C_COMPILER MATCHES "clang")
    message(WARNING "Consider using clang for better fuzzing support")
endif()

# Source files - adjust these paths to match your project structure
set(SRC_DIR "../src")
set(FUZZ_SRC "${SRC_DIR}/{{target_name}}.c")
set(DRIVER_SRC "driver/main.c")

# Include directories - adjust to match your project
include_directories(${SRC_DIR} ../include)

# Common compiler flags
set(COMMON_FLAGS "-g -O2 -Wall -Wextra")

# Define fuzzer type macros
add_compile_definitions(TARGET_NAME="{{target_name}}")

# Standalone fuzzer (no special fuzzer engine required)
add_executable({{target_name}}-standalone
    ${FUZZ_SRC}
    ${DRIVER_SRC}
)
target_compile_definitions({{target_name}}-standalone PRIVATE FUZZER_TYPE_STANDALONE)
target_compile_options({{target_name}}-standalone PRIVATE ${COMMON_FLAGS})

# AFL fuzzer with persistent mode
find_program(AFL_CLANG_FAST afl-clang-fast)
if(AFL_CLANG_FAST)
    add_executable({{target_name}}-afl
        ${FUZZ_SRC}
        ${DRIVER_SRC}
    )
    target_compile_definitions({{target_name}}-afl PRIVATE FUZZER_TYPE_AFL)
    set_target_properties({{target_name}}-afl PROPERTIES
        C_COMPILER ${AFL_CLANG_FAST}
        COMPILE_FLAGS ${COMMON_FLAGS}
    )
else()
    message(STATUS "AFL not found, skipping AFL target")
endif()

# libFuzzer (requires clang with -fsanitize=fuzzer support)
if(CMAKE_C_COMPILER MATCHES "clang")
    add_executable({{target_name}}-libfuzzer
        ${FUZZ_SRC}
        ${DRIVER_SRC}
    )
    target_compile_definitions({{target_name}}-libfuzzer PRIVATE FUZZER_TYPE_LIBFUZZER)
    target_compile_options({{target_name}}-libfuzzer PRIVATE 
        ${COMMON_FLAGS} -fsanitize=fuzzer,address
    )
    target_link_options({{target_name}}-libfuzzer PRIVATE -fsanitize=fuzzer,address)
else()
    message(STATUS "libFuzzer requires clang, skipping libFuzzer target")
endif()

# HonggFuzz
find_program(HFUZZ_CLANG hfuzz-clang)
if(HFUZZ_CLANG)
    add_executable({{target_name}}-honggfuzz
        ${FUZZ_SRC}
        ${DRIVER_SRC}
    )
    target_compile_definitions({{target_name}}-honggfuzz PRIVATE FUZZER_TYPE_HONGGFUZZ)
    set_target_properties({{target_name}}-honggfuzz PROPERTIES
        C_COMPILER ${HFUZZ_CLANG}
        COMPILE_FLAGS ${COMMON_FLAGS}
    )
else()
    message(STATUS "HonggFuzz not found, skipping HonggFuzz target")
endif()

# Custom targets for running fuzzers

# Run standalone fuzzer
add_custom_target(run-standalone
    COMMAND ./{{target_name}}-standalone testcases/
    DEPENDS {{target_name}}-standalone
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "Running standalone fuzzer"
)

# Run AFL fuzzer
if(AFL_CLANG_FAST)
    find_program(AFL_FUZZ afl-fuzz)
    if(AFL_FUZZ)
        add_custom_target(run-afl
            COMMAND ${CMAKE_COMMAND} -E make_directory findings
            COMMAND ${AFL_FUZZ} -i testcases -o findings -- ./{{target_name}}-afl
            DEPENDS {{target_name}}-afl
            WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
            COMMENT "Running AFL fuzzer"
        )
    endif()
endif()

# Run libFuzzer
if(CMAKE_C_COMPILER MATCHES "clang")
    add_custom_target(run-libfuzzer
        COMMAND ${CMAKE_COMMAND} -E make_directory corpus
        COMMAND ./{{target_name}}-libfuzzer corpus/ -dict=dictionaries/{{target_name}}.dict
        DEPENDS {{target_name}}-libfuzzer
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
        COMMENT "Running libFuzzer"
    )
endif()

# Run HonggFuzz
if(HFUZZ_CLANG)
    find_program(HONGGFUZZ honggfuzz)
    if(HONGGFUZZ)
        add_custom_target(run-honggfuzz
            COMMAND ${CMAKE_COMMAND} -E make_directory corpus
            COMMAND ${HONGGFUZZ} -i testcases/ -W corpus/ -- ./{{target_name}}-honggfuzz
            DEPENDS {{target_name}}-honggfuzz
            WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
            COMMENT "Running HonggFuzz"
        )
    endif()
endif()

# Test target - builds and runs basic smoke test
add_custom_target(test
    COMMAND echo "FUZZ" | ./{{target_name}}-standalone
    DEPENDS {{target_name}}-standalone
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "Running smoke test"
)

# Install fuzzer engines (system-specific)
add_custom_target(install-deps
    COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/install-deps.sh
    COMMENT "Installing fuzzing dependencies"
)

# Copy necessary files to build directory
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/testcases/example.txt 
               ${CMAKE_CURRENT_BINARY_DIR}/testcases/example.txt COPYONLY)
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/dictionaries/{{target_name}}.dict 
               ${CMAKE_CURRENT_BINARY_DIR}/dictionaries/{{target_name}}.dict COPYONLY)

# Print build information
message(STATUS "Fuzzing targets configured:")
message(STATUS "  - {{target_name}}-standalone (always available)")
if(AFL_CLANG_FAST)
    message(STATUS "  - {{target_name}}-afl (AFL available)")
endif()
if(CMAKE_C_COMPILER MATCHES "clang")
    message(STATUS "  - {{target_name}}-libfuzzer (clang available)")
endif()
if(HFUZZ_CLANG)
    message(STATUS "  - {{target_name}}-honggfuzz (HonggFuzz available)")
endif()

# Usage instructions
message(STATUS "")
message(STATUS "Build with: cmake --build . --target <fuzzer-name>")
message(STATUS "Run with: cmake --build . --target run-<fuzzer-name>")
message(STATUS "Test with: cmake --build . --target test")
message(STATUS "")