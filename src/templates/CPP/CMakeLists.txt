cmake_minimum_required(VERSION 3.16)
# Convert project name with slashes to valid CMake identifier
string(REPLACE "/" "_" CMAKE_PROJECT_NAME "{{project_name}}")
project(${CMAKE_PROJECT_NAME} C CXX)

# =============================================================================
# Compiler Standards and Detection
# =============================================================================

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 11)

# Detect fuzzing tools availability
find_program(CLANG_CXX clang++ DOC "Clang C++ compiler for libFuzzer")
find_program(AFL_CXX NAMES afl-clang-fast++ afl-clang++ DOC "AFL C++ compiler wrapper") 
find_program(HFUZZ_CXX hfuzz-clang++ DOC "HonggFuzz C++ compiler wrapper")

# Set boolean flags for easier conditional logic
if(CLANG_CXX)
    set(HAVE_CLANG TRUE)
    message(STATUS "Found clang++: ${CLANG_CXX}")
else()
    set(HAVE_CLANG FALSE)
    message(STATUS "clang++ not found - libFuzzer support disabled")
endif()

if(AFL_CXX)
    set(HAVE_AFL TRUE)
    message(STATUS "Found AFL++: ${AFL_CXX}")
else()
    set(HAVE_AFL FALSE)
endif()

if(HFUZZ_CXX)
    set(HAVE_HFUZZ TRUE)
    message(STATUS "Found HonggFuzz: ${HFUZZ_CXX}")
else()
    set(HAVE_HFUZZ FALSE)
endif()

# =============================================================================
# Compiler Configuration
# =============================================================================

# Standard compiler flags for normal builds
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -g")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -g")
set(CMAKE_C_FLAGS_DEBUG "-O0")
set(CMAKE_C_FLAGS_RELEASE "-O2")
set(CMAKE_CXX_FLAGS_DEBUG "-O0")
set(CMAKE_CXX_FLAGS_RELEASE "-O2")

# Fuzzing compiler flags (with sanitizers when clang available)
if(HAVE_CLANG)
    set(FUZZ_CXX_FLAGS -g -O1 -fsanitize=address,undefined)
    set(FUZZ_MODE "instrumented")
else()
    set(FUZZ_CXX_FLAGS -g -O1)
    set(FUZZ_MODE "basic")
endif()

# =============================================================================
# Source Files and Directories
# =============================================================================

set(LIB_SOURCES src/lib.cpp)

# Create safe target names (replace slashes with underscores)
string(REPLACE "/" "_" PROJECT_TARGET_NAME "{{project_name}}")

# =============================================================================
# Standard Library Target (no instrumentation)
# =============================================================================

add_library(${PROJECT_TARGET_NAME} STATIC ${LIB_SOURCES})
target_include_directories(${PROJECT_TARGET_NAME} PUBLIC include)
target_compile_options(${PROJECT_TARGET_NAME} PRIVATE -O0)

# =============================================================================
# Instrumented Library for Fuzzing
# =============================================================================

add_library(${PROJECT_TARGET_NAME}_fuzz STATIC ${LIB_SOURCES})
target_include_directories(${PROJECT_TARGET_NAME}_fuzz PUBLIC include)

# Use clang++ with sanitizers if available, otherwise basic flags
if(HAVE_CLANG)
    set_target_properties(${PROJECT_TARGET_NAME}_fuzz PROPERTIES
        CXX_COMPILER ${CLANG_CXX}
    )
    target_compile_options(${PROJECT_TARGET_NAME}_fuzz PRIVATE ${FUZZ_CXX_FLAGS})
else()
    target_compile_options(${PROJECT_TARGET_NAME}_fuzz PRIVATE ${FUZZ_CXX_FLAGS})
endif()

# =============================================================================
# Standalone Library (no sanitizers)
# =============================================================================

add_library(${PROJECT_TARGET_NAME}_standalone STATIC ${LIB_SOURCES})
target_include_directories(${PROJECT_TARGET_NAME}_standalone PUBLIC include)
target_compile_options(${PROJECT_TARGET_NAME}_standalone PRIVATE -O1)

# =============================================================================
# Main Executable
# =============================================================================

add_executable(${PROJECT_TARGET_NAME}-app src/main.cpp)
target_link_libraries(${PROJECT_TARGET_NAME}-app ${PROJECT_TARGET_NAME})

# =============================================================================
# Testing Targets
# =============================================================================

{{#unless minimal}}
# Add test subdirectory (unit tests)
add_subdirectory(test)

# Custom target for integration testing with sample data
add_custom_target(integration-test
    COMMAND {{target_name}} test_data/valid.nmea
    COMMAND {{target_name}} test_data/oob_read.nmea
    DEPENDS {{target_name}}
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running integration tests with sample data"
)

# Copy test data to build directory
add_custom_target(test_data
    COMMAND ${CMAKE_COMMAND} -E copy_directory 
        ${CMAKE_SOURCE_DIR}/test_data ${CMAKE_BINARY_DIR}/test_data
    COMMENT "Copying test data to build directory"
)
add_dependencies(integration-test test_data)
{{/unless}}

# =============================================================================
# Fuzzing Subdirectory
# =============================================================================

# Add fuzz subdirectory for fuzzing targets
add_subdirectory(fuzz)

# =============================================================================
# Universal Fuzzing Targets (delegating to fuzz subdirectory)
# =============================================================================

# Main fuzzing target - builds all compatible fuzzers
add_custom_target(fuzz
    COMMENT "Building all compatible fuzzing targets"
)

# Individual fuzzer targets (delegate to fuzz subdirectory)
add_custom_target(fuzz-libfuzzer
    COMMAND ${CMAKE_COMMAND} --build . --target libfuzzer
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/fuzz
    COMMENT "Building libFuzzer target"
)
add_dependencies(fuzz-libfuzzer ${PROJECT_TARGET_NAME}_fuzz)

add_custom_target(fuzz-afl
    COMMAND ${CMAKE_COMMAND} --build . --target afl
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/fuzz
    COMMENT "Building AFL target"
)
add_dependencies(fuzz-afl ${PROJECT_TARGET_NAME}_fuzz)

add_custom_target(fuzz-honggfuzz
    COMMAND ${CMAKE_COMMAND} --build . --target honggfuzz
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/fuzz
    COMMENT "Building HonggFuzz target"
)
add_dependencies(fuzz-honggfuzz ${PROJECT_TARGET_NAME}_fuzz)

add_custom_target(fuzz-standalone
    COMMAND ${CMAKE_COMMAND} --build . --target standalone
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/fuzz
    COMMENT "Building standalone target"
)
add_dependencies(fuzz-standalone ${PROJECT_TARGET_NAME}_standalone)

# Add dependencies to main fuzz target based on detected capabilities
if(HAVE_CLANG)
    add_dependencies(fuzz fuzz-libfuzzer)
endif()
if(HAVE_AFL OR HAVE_CLANG)
    add_dependencies(fuzz fuzz-afl)
endif()
if(HAVE_HFUZZ)
    add_dependencies(fuzz fuzz-honggfuzz)
endif()
add_dependencies(fuzz fuzz-standalone)

# =============================================================================
# Information and Help Targets
# =============================================================================

# Target to show fuzzing environment information
add_custom_target(fuzz-info
    COMMAND ${CMAKE_COMMAND} -E echo "🔍 Fuzzing environment detected:"
    COMMAND ${CMAKE_COMMAND} -E echo "   Build type: ${CMAKE_BUILD_TYPE}"
    COMMAND ${CMAKE_COMMAND} -E echo "   CXX compiler: ${CMAKE_CXX_COMPILER}"
    COMMAND ${CMAKE_COMMAND} -E echo "   Fuzz mode: ${FUZZ_MODE}"
    COMMAND ${CMAKE_COMMAND} -E echo "   clang++: ${HAVE_CLANG}"
    COMMAND ${CMAKE_COMMAND} -E echo "   AFL++: ${HAVE_AFL}"  
    COMMAND ${CMAKE_COMMAND} -E echo "   HonggFuzz: ${HAVE_HFUZZ}"
    COMMENT "Displaying fuzzing environment information"
)

# Help target
add_custom_target(fuzz-help
    COMMAND ${CMAKE_COMMAND} -E echo "Universal CMake build for {{project_name}} - Environment-driven fuzzing"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Main targets:"
    COMMAND ${CMAKE_COMMAND} -E echo "  all                  - Build normal library and executable"
    COMMAND ${CMAKE_COMMAND} -E echo "  ${PROJECT_TARGET_NAME}               - Build normal library"
    COMMAND ${CMAKE_COMMAND} -E echo "  ${PROJECT_TARGET_NAME}_fuzz         - Build instrumented library"
    COMMAND ${CMAKE_COMMAND} -E echo "  ${PROJECT_TARGET_NAME}_standalone   - Build standalone library"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Fuzzing targets:"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz                 - Build all compatible fuzzers"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-libfuzzer       - Build libFuzzer target"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-afl             - Build AFL target"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-honggfuzz       - Build HonggFuzz target"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-standalone      - Build standalone target"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Utilities:"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-info            - Show detected environment"
{{#unless minimal}}
    COMMAND ${CMAKE_COMMAND} -E echo "  test                 - Run unit tests"
    COMMAND ${CMAKE_COMMAND} -E echo "  integration-test     - Run integration tests"
{{/unless}}
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-help            - Show this help"
    COMMENT "Displaying help information"
)