# CMake configuration for fuzzing {{project_name}} with {{default_fuzzer}}
# This CMakeLists.txt is optimized for your chosen fuzzer and integration

cmake_minimum_required(VERSION 3.16)

{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer requires clang - auto-detect if not specified
if(NOT DEFINED CMAKE_C_COMPILER)
    find_program(CLANG_C clang
        DOC "Clang C compiler for libFuzzer")
    if(CLANG_C)
        set(CMAKE_C_COMPILER ${CLANG_C})
        message(STATUS "Auto-detected clang for libFuzzer: ${CLANG_C}")
    else()
        message(FATAL_ERROR "libFuzzer requires clang. Please install clang or set CMAKE_C_COMPILER=clang")
    endif()
endif()
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# AFL configuration - find best available AFL++ C compiler
if(NOT DEFINED CMAKE_C_COMPILER)
    find_program(AFL_C_COMPILER 
        NAMES afl-clang-fast afl-clang afl-gcc
        DOC "AFL++ C compiler wrapper")
    if(AFL_C_COMPILER)
        set(CMAKE_C_COMPILER ${AFL_C_COMPILER})
        message(STATUS "Auto-detected AFL compiler: ${AFL_C_COMPILER}")
    else()
        message(FATAL_ERROR "AFL fuzzing requires AFL++. Please install AFL++ or set CMAKE_C_COMPILER=afl-clang-fast")
    endif()
endif()
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# HonggFuzz configuration - find best available HonggFuzz compiler
if(NOT DEFINED CMAKE_C_COMPILER)
    find_program(HFUZZ_C_COMPILER 
        NAMES hfuzz-clang clang
        DOC "HonggFuzz C compiler")
    if(HFUZZ_C_COMPILER)
        set(CMAKE_C_COMPILER ${HFUZZ_C_COMPILER})
        message(STATUS "Auto-detected HonggFuzz compiler: ${HFUZZ_C_COMPILER}")
    else()
        message(FATAL_ERROR "HonggFuzz requires clang. Please install HonggFuzz or clang")
    endif()
endif()
{{/if}}

project({{project_name}}-fuzz-{{default_fuzzer}})

# Set C standard
set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Set default build type to Debug for fuzzing
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

{{#if (eq default_fuzzer "libfuzzer")}}
# Verify libFuzzer compiler was detected correctly
if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    message(FATAL_ERROR "libFuzzer requires clang, but got ${CMAKE_C_COMPILER_ID} compiler: ${CMAKE_C_COMPILER}")
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
# Verify AFL compiler was detected correctly
if(NOT CMAKE_C_COMPILER_ID MATCHES "AFL|Clang")
    message(FATAL_ERROR "AFL requires AFL++ or clang, but got ${CMAKE_C_COMPILER_ID} compiler: ${CMAKE_C_COMPILER}")
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
# Verify HonggFuzz compiler was detected correctly
if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    message(FATAL_ERROR "HonggFuzz requires clang, but got ${CMAKE_C_COMPILER_ID} compiler: ${CMAKE_C_COMPILER}")
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

# Smart library detection for existing projects
{{#if (eq integration "cmake")}}
# For CMake integration, intelligently detect existing targets
set(LIBRARY_TARGET_FOUND FALSE)

# Helper function to validate target type (libraries only)
function(is_library_target target_name result_var)
    if(TARGET ${target_name})
        get_target_property(target_type ${target_name} TYPE)
        if(target_type STREQUAL "STATIC_LIBRARY" OR target_type STREQUAL "SHARED_LIBRARY")
            set(${result_var} TRUE PARENT_SCOPE)
        else()
            set(${result_var} FALSE PARENT_SCOPE)
        endif()
    else()
        set(${result_var} FALSE PARENT_SCOPE)
    endif()
endfunction()

# Priority 1: Check for fuzzer-specific library (ideal for full mode)
set(FUZZER_SPECIFIC_TARGET "{{project_name}}-{{default_fuzzer}}")
is_library_target(${FUZZER_SPECIFIC_TARGET} IS_FUZZER_LIB)
if(IS_FUZZER_LIB)
    set(REQUIRED_LIBRARY_TARGET "${FUZZER_SPECIFIC_TARGET}")
    set(LIBRARY_TARGET_FOUND TRUE)
    message(STATUS "✓ Found fuzzer-specific library: ${REQUIRED_LIBRARY_TARGET}")

# Priority 2: Check for general library naming patterns (libraries only)
elseif(TARGET {{project_name}}_lib)
    is_library_target("{{project_name}}_lib" IS_LIB)
    if(IS_LIB)
        set(REQUIRED_LIBRARY_TARGET "{{project_name}}_lib")
        set(LIBRARY_TARGET_FOUND TRUE)
        message(STATUS "✓ Found library target: ${REQUIRED_LIBRARY_TARGET}")
    endif()
elseif(TARGET lib{{project_name}})
    is_library_target("lib{{project_name}}" IS_LIB)
    if(IS_LIB)
        set(REQUIRED_LIBRARY_TARGET "lib{{project_name}}")
        set(LIBRARY_TARGET_FOUND TRUE)
        message(STATUS "✓ Found library target: ${REQUIRED_LIBRARY_TARGET}")
    endif()

# Priority 3: Check project name (but only if it's a library, not executable)
elseif(TARGET {{project_name}})
    is_library_target("{{project_name}}" IS_LIB)
    if(IS_LIB)
        set(REQUIRED_LIBRARY_TARGET "{{project_name}}")
        set(LIBRARY_TARGET_FOUND TRUE)
        message(STATUS "✓ Found project library: ${REQUIRED_LIBRARY_TARGET}")
    else()
        message(STATUS "Found {{project_name}} but it's not a library (skipping)")
    endif()
endif()

# Priority 4: Auto-detect from available libraries
if(NOT LIBRARY_TARGET_FOUND)
    # Get all targets and filter for libraries only
    get_property(all_targets DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR} PROPERTY BUILDSYSTEM_TARGETS)
    set(library_targets)
    foreach(target ${all_targets})
        get_target_property(target_type ${target} TYPE)
        if(target_type STREQUAL "STATIC_LIBRARY" OR target_type STREQUAL "SHARED_LIBRARY")
            list(APPEND library_targets ${target})
        endif()
    endforeach()
    
    # If we found exactly one library, use it
    list(LENGTH library_targets num_libraries)
    if(num_libraries EQUAL 1)
        list(GET library_targets 0 REQUIRED_LIBRARY_TARGET)
        set(LIBRARY_TARGET_FOUND TRUE)
        message(STATUS "✓ Auto-detected library target: ${REQUIRED_LIBRARY_TARGET}")
    elseif(num_libraries GREATER 1)
        message(STATUS "Multiple library targets found: ${library_targets}")
        # Try to pick the best one (fuzzer-specific or main library)
        foreach(lib_target ${library_targets})
            if(lib_target MATCHES ".*-{{default_fuzzer}}$")
                set(REQUIRED_LIBRARY_TARGET "${lib_target}")
                set(LIBRARY_TARGET_FOUND TRUE)
                message(STATUS "✓ Selected fuzzer-specific library: ${REQUIRED_LIBRARY_TARGET}")
                break()
            endif()
        endforeach()
        if(NOT LIBRARY_TARGET_FOUND)
            list(GET library_targets 0 REQUIRED_LIBRARY_TARGET)
            set(LIBRARY_TARGET_FOUND TRUE)
            message(STATUS "✓ Selected first library target: ${REQUIRED_LIBRARY_TARGET}")
            message(STATUS "Other available libraries: ${library_targets}")
        endif()
    endif()
endif()

# If no target found, fall back to the expected fuzzing library name
if(NOT LIBRARY_TARGET_FOUND)
    set(REQUIRED_LIBRARY_TARGET "${LIBRARY_NAME}")
    message(WARNING "No existing library target found. Expecting: ${REQUIRED_LIBRARY_TARGET}")
    message(STATUS "If your library has a different name, please edit REQUIRED_LIBRARY_TARGET in this CMakeLists.txt")
endif()
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