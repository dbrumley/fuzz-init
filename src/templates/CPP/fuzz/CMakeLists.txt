# Universal CMake fuzzing configuration for {{project_name}}
# Links against standard lib{{project_name}}.a with intelligent fuzzer detection

cmake_minimum_required(VERSION 3.16)
# Convert project name with slashes to valid CMake identifier  
string(REPLACE "/" "_" CMAKE_PROJECT_NAME "{{project_name}}")
project(${CMAKE_PROJECT_NAME}-fuzz C CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# =============================================================================
# Environment Detection
# =============================================================================

# Detect fuzzing tools availability
find_program(CLANG_CXX clang++ DOC "Clang C++ compiler for libFuzzer")
find_program(AFL_CXX NAMES afl-clang-fast++ afl-clang++ DOC "AFL C++ compiler wrapper")
find_program(HFUZZ_CXX hfuzz-clang++ DOC "HonggFuzz C++ compiler wrapper")

# Set boolean flags
if(CLANG_CXX)
    set(HAVE_CLANG TRUE)
else()
    set(HAVE_CLANG FALSE)
endif()

if(AFL_CXX)
    set(HAVE_AFL TRUE)
else()
    set(HAVE_AFL FALSE)
endif()

if(HFUZZ_CXX)
    set(HAVE_HFUZZ TRUE)
else()
    set(HAVE_HFUZZ FALSE)
endif()

# =============================================================================
# Library Path Configuration
# =============================================================================

{{#unless minimal}}
# Create safe target names (replace slashes with underscores)
string(REPLACE "/" "_" PROJECT_TARGET_NAME "{{project_name}}")

# Full mode - library targets should be available from parent
set(INCLUDES ../include)
set(LIBRARY_TARGET ${PROJECT_TARGET_NAME}_fuzz)
set(STANDALONE_TARGET ${PROJECT_TARGET_NAME}_standalone)
set(INTEGRATION_MODE "library")
{{else}}
# Create safe target names (replace slashes with underscores)
string(REPLACE "/" "_" PROJECT_TARGET_NAME "{{project_name}}")

# Minimal mode - try to discover user's existing library
set(INCLUDES ./include ../include)

# Try to find user's library files
file(GLOB_RECURSE DETECTED_LIBS 
    RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/..
    "../lib*.a"
    "../build/lib*.a"
    "../build/*/lib*.a"
)

# Filter out fuzz directory libraries
list(FILTER DETECTED_LIBS EXCLUDE REGEX "fuzz/")

if(DETECTED_LIBS)
    list(GET DETECTED_LIBS 0 DETECTED_LIB)
    get_filename_component(DETECTED_LIB_PATH "../${DETECTED_LIB}" ABSOLUTE)
    get_filename_component(DETECTED_LIB_NAME ${DETECTED_LIB} NAME_WE)
    set(INTEGRATION_MODE "library")
    message(STATUS "Found user library: ${DETECTED_LIB_PATH}")
else()
    set(INTEGRATION_MODE "standalone")
    message(STATUS "No user library found - using standalone mode")
endif()
{{/unless}}

# =============================================================================
# Source Files
# =============================================================================

set(FUZZ_HARNESS_SRC src/{{target_name}}.cpp)
set(DRIVER_SRC driver/main.c)

# =============================================================================
# Include Directories
# =============================================================================

include_directories(${INCLUDES})

# =============================================================================
# Compiler Configuration
# =============================================================================

# Common flags for fuzzer harnesses
set(COMMON_CXX_FLAGS -g -O2 -Wall -Wextra)
set(COMMON_C_FLAGS -g -O2 -Wall -Wextra)

# Sanitizer flags when clang is available
if(HAVE_CLANG)
    set(SANITIZER_FLAGS -fsanitize=address,undefined)
else()
    set(SANITIZER_FLAGS "")
endif()

# =============================================================================
# libFuzzer Target
# =============================================================================

add_executable(${PROJECT_TARGET_NAME}-libfuzzer ${FUZZ_HARNESS_SRC})

if(HAVE_CLANG)
    # Use clang++ for libFuzzer (required)
    set_target_properties(${PROJECT_TARGET_NAME}-libfuzzer PROPERTIES
        CXX_COMPILER ${CLANG_CXX}
    )
    
    target_compile_options(${PROJECT_TARGET_NAME}-libfuzzer PRIVATE 
        ${COMMON_CXX_FLAGS} ${SANITIZER_FLAGS}
    )
    
    target_link_options(${PROJECT_TARGET_NAME}-libfuzzer PRIVATE 
        ${SANITIZER_FLAGS} -fsanitize=fuzzer
    )
    
    # Link against appropriate library
{{#unless minimal}}
    if(TARGET ${LIBRARY_TARGET})
        target_link_libraries(${PROJECT_TARGET_NAME}-libfuzzer ${LIBRARY_TARGET})
    else()
        message(FATAL_ERROR "Library target ${LIBRARY_TARGET} not found. Build parent project first.")
    endif()
{{else}}
    if(INTEGRATION_MODE STREQUAL "library" AND EXISTS ${DETECTED_LIB_PATH})
        target_link_libraries(${PROJECT_TARGET_NAME}-libfuzzer ${DETECTED_LIB_PATH})
    endif()
{{/unless}}
    
    message(STATUS "✅ libFuzzer target configured")
else()
    # No clang++ found - create a target that fails with helpful message
    add_custom_target(libfuzzer-error
        COMMAND ${CMAKE_COMMAND} -E echo "❌ libFuzzer requires clang++ but only found: ${CMAKE_CXX_COMPILER}"
        COMMAND ${CMAKE_COMMAND} -E echo "💡 Install clang: sudo apt install clang (Ubuntu) or xcode-select --install (macOS)"
        COMMAND ${CMAKE_COMMAND} -E false
        COMMENT "libFuzzer requires clang++"
    )
    add_dependencies(${PROJECT_TARGET_NAME}-libfuzzer libfuzzer-error)
endif()

# Create libfuzzer alias target
add_custom_target(libfuzzer DEPENDS ${PROJECT_TARGET_NAME}-libfuzzer)

# =============================================================================
# AFL Target
# =============================================================================

add_executable(${PROJECT_TARGET_NAME}-afl ${FUZZ_HARNESS_SRC} ${DRIVER_SRC})

# Choose AFL compiler: AFL++ > clang++ > g++
if(HAVE_AFL)
    set(AFL_COMPILER ${AFL_CXX})
    set(AFL_C_COMPILER afl-clang-fast)
elseif(HAVE_CLANG)
    set(AFL_COMPILER ${CLANG_CXX})
    set(AFL_C_COMPILER clang)
else()
    set(AFL_COMPILER ${CMAKE_CXX_COMPILER})
    set(AFL_C_COMPILER ${CMAKE_C_COMPILER})
endif()

set_target_properties(${PROJECT_TARGET_NAME}-afl PROPERTIES
    CXX_COMPILER ${AFL_COMPILER}
    C_COMPILER ${AFL_C_COMPILER}
)

target_compile_options(${PROJECT_TARGET_NAME}-afl PRIVATE 
    ${COMMON_CXX_FLAGS} ${SANITIZER_FLAGS}
)

if(SANITIZER_FLAGS)
    target_link_options(${PROJECT_TARGET_NAME}-afl PRIVATE ${SANITIZER_FLAGS})
endif()

# Link against appropriate library
{{#unless minimal}}
if(TARGET ${LIBRARY_TARGET})
    target_link_libraries(${PROJECT_TARGET_NAME}-afl ${LIBRARY_TARGET})
elseif(TARGET ${STANDALONE_TARGET})
    target_link_libraries(${PROJECT_TARGET_NAME}-afl ${STANDALONE_TARGET})
    message(STATUS "⚠️  Linking AFL with standalone library (no sanitizers)")
else()
    message(FATAL_ERROR "No library target available for AFL")
endif()
{{else}}
if(INTEGRATION_MODE STREQUAL "library" AND EXISTS ${DETECTED_LIB_PATH})
    target_link_libraries(${PROJECT_TARGET_NAME}-afl ${DETECTED_LIB_PATH})
endif()
{{/unless}}

add_custom_target(afl DEPENDS ${PROJECT_TARGET_NAME}-afl)
message(STATUS "✅ AFL target configured")

# =============================================================================
# HonggFuzz Target
# =============================================================================

if(HAVE_HFUZZ)
    add_executable(${PROJECT_TARGET_NAME}-honggfuzz ${FUZZ_HARNESS_SRC} ${DRIVER_SRC})
    
    set_target_properties(${PROJECT_TARGET_NAME}-honggfuzz PROPERTIES
        CXX_COMPILER ${HFUZZ_CXX}
    )
    
    target_compile_options(${PROJECT_TARGET_NAME}-honggfuzz PRIVATE 
        ${COMMON_CXX_FLAGS} ${SANITIZER_FLAGS}
    )
    
    if(SANITIZER_FLAGS)
        target_link_options(${PROJECT_TARGET_NAME}-honggfuzz PRIVATE ${SANITIZER_FLAGS})
    endif()
    
    # Link against appropriate library
{{#unless minimal}}
    if(TARGET ${LIBRARY_TARGET})
        target_link_libraries(${PROJECT_TARGET_NAME}-honggfuzz ${LIBRARY_TARGET})
    else()
        message(FATAL_ERROR "Library target ${LIBRARY_TARGET} not found")
    endif()
{{else}}
    if(INTEGRATION_MODE STREQUAL "library" AND EXISTS ${DETECTED_LIB_PATH})
        target_link_libraries(${PROJECT_TARGET_NAME}-honggfuzz ${DETECTED_LIB_PATH})
    endif()
{{/unless}}
    
    add_custom_target(honggfuzz DEPENDS ${PROJECT_TARGET_NAME}-honggfuzz)
    message(STATUS "✅ HonggFuzz target configured")
else()
    # HonggFuzz not found - create error target
    add_custom_target(honggfuzz
        COMMAND ${CMAKE_COMMAND} -E echo "❌ HonggFuzz not found. Install: apt install honggfuzz"
        COMMAND ${CMAKE_COMMAND} -E false
        COMMENT "HonggFuzz not available"
    )
endif()

# =============================================================================
# Standalone Target
# =============================================================================

add_executable(${PROJECT_TARGET_NAME}-standalone ${FUZZ_HARNESS_SRC} ${DRIVER_SRC})

target_compile_options(${PROJECT_TARGET_NAME}-standalone PRIVATE ${COMMON_CXX_FLAGS})

# Link against standalone library (no sanitizers)
{{#unless minimal}}
if(TARGET ${STANDALONE_TARGET})
    target_link_libraries(${PROJECT_TARGET_NAME}-standalone ${STANDALONE_TARGET})
else()
    message(FATAL_ERROR "Standalone library target ${STANDALONE_TARGET} not found")
endif()
{{else}}
if(INTEGRATION_MODE STREQUAL "library" AND EXISTS ${DETECTED_LIB_PATH})
    target_link_libraries(${PROJECT_TARGET_NAME}-standalone ${DETECTED_LIB_PATH})
endif()
{{/unless}}

add_custom_target(standalone DEPENDS ${PROJECT_TARGET_NAME}-standalone)
message(STATUS "✅ Standalone target configured")

# =============================================================================
# Testing Targets
# =============================================================================

# Test target - runs the best available fuzzer
if(HAVE_CLANG)
    set(TEST_TARGET ${PROJECT_TARGET_NAME}-libfuzzer)
    set(TEST_COMMAND "${TEST_TARGET} testsuite/ -runs=10 -max_total_time=5")
else()
    set(TEST_TARGET ${PROJECT_TARGET_NAME}-standalone)
    set(TEST_COMMAND "echo 'test input' | ${TEST_TARGET}")
endif()

add_custom_target(test
    COMMAND ${CMAKE_COMMAND} -E echo "🧪 Testing ${TEST_TARGET}..."
    COMMAND ${TEST_COMMAND}
    COMMAND ${CMAKE_COMMAND} -E echo "✅ Test passed!"
    DEPENDS ${TEST_TARGET}
    COMMENT "Running fuzzer tests"
)

# Individual test targets
add_custom_target(test-libfuzzer
    COMMAND ${PROJECT_TARGET_NAME}-libfuzzer testsuite/ -runs=10 -max_total_time=5
    DEPENDS ${PROJECT_TARGET_NAME}-libfuzzer
    COMMENT "Testing libFuzzer"
)

add_custom_target(test-afl
    COMMAND ${CMAKE_COMMAND} -E echo "test input" | ${PROJECT_TARGET_NAME}-afl
    DEPENDS ${PROJECT_TARGET_NAME}-afl
    COMMENT "Testing AFL target"
)

add_custom_target(test-standalone
    COMMAND ${CMAKE_COMMAND} -E echo "test input" | ${PROJECT_TARGET_NAME}-standalone
    DEPENDS ${PROJECT_TARGET_NAME}-standalone
    COMMENT "Testing standalone target"
)

# =============================================================================
# Environment Information
# =============================================================================

add_custom_target(check-environment
    COMMAND ${CMAKE_COMMAND} -E echo "🔍 Fuzzing environment:"
    COMMAND ${CMAKE_COMMAND} -E echo "   Mode: {{#if minimal}}minimal{{else}}full{{/if}}"
    COMMAND ${CMAKE_COMMAND} -E echo "   CXX Compiler: ${CMAKE_CXX_COMPILER}"
    COMMAND ${CMAKE_COMMAND} -E echo "   Integration: ${INTEGRATION_MODE}"
{{#unless minimal}}
    COMMAND ${CMAKE_COMMAND} -E echo "   Library: ${LIBRARY_TARGET}"
{{else}}
    COMMAND ${CMAKE_COMMAND} -E echo "   Detected lib: ${DETECTED_LIB_PATH}"
{{/unless}}
    COMMAND ${CMAKE_COMMAND} -E echo "   clang++: ${HAVE_CLANG}"
    COMMAND ${CMAKE_COMMAND} -E echo "   AFL++: ${HAVE_AFL}"
    COMMAND ${CMAKE_COMMAND} -E echo "   HonggFuzz: ${HAVE_HFUZZ}"
    COMMENT "Displaying environment information"
)

# =============================================================================
# Library Validation
# =============================================================================

add_custom_target(check-library
{{#unless minimal}}
    COMMAND ${CMAKE_COMMAND} -E echo "✅ Full mode: Using parent project libraries"
{{else}}
    COMMAND ${CMAKE_COMMAND} -E echo "ℹ️  Minimal mode library check:"
    COMMAND ${CMAKE_COMMAND} -E echo "   Integration: ${INTEGRATION_MODE}"
    COMMAND ${CMAKE_COMMAND} -E echo "   Detected: ${DETECTED_LIB_PATH}"
    COMMAND ${CMAKE_COMMAND} -E echo "🔄 To integrate with your library:"
    COMMAND ${CMAKE_COMMAND} -E echo "   1. Build your library with sanitizer flags"
    COMMAND ${CMAKE_COMMAND} -E echo "   2. Place it where we can find it (../lib{{project_name}}.a)"
    COMMAND ${CMAKE_COMMAND} -E echo "   3. Edit src/{{target_name}}.cpp to call your functions"
{{/unless}}
    COMMENT "Checking library configuration"
)

# =============================================================================
# Help Target
# =============================================================================

add_custom_target(fuzz-help-local
    COMMAND ${CMAKE_COMMAND} -E echo "Universal CMake Fuzzing for {{project_name}}"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Quick start:"
    COMMAND ${CMAKE_COMMAND} -E echo "  cmake --build . --target libfuzzer  - Build best fuzzer"
    COMMAND ${CMAKE_COMMAND} -E echo "  cmake --build . --target test       - Build and test"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Fuzzer targets:"
    COMMAND ${CMAKE_COMMAND} -E echo "  libfuzzer    - Build libFuzzer target (requires clang++)"
    COMMAND ${CMAKE_COMMAND} -E echo "  afl          - Build AFL target"
    COMMAND ${CMAKE_COMMAND} -E echo "  honggfuzz    - Build HonggFuzz target (requires honggfuzz)"
    COMMAND ${CMAKE_COMMAND} -E echo "  standalone   - Build standalone binary"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Testing:"
    COMMAND ${CMAKE_COMMAND} -E echo "  test         - Test best available fuzzer"
    COMMAND ${CMAKE_COMMAND} -E echo "  test-libfuzzer - Test libFuzzer specifically"
    COMMAND ${CMAKE_COMMAND} -E echo "  test-afl     - Test AFL specifically"
    COMMAND ${CMAKE_COMMAND} -E echo "  test-standalone - Test standalone specifically"
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Utilities:"
    COMMAND ${CMAKE_COMMAND} -E echo "  check-environment - Show detected environment"
    COMMAND ${CMAKE_COMMAND} -E echo "  check-library     - Verify library setup"
    COMMAND ${CMAKE_COMMAND} -E echo "  fuzz-help-local  - Show this help"
{{#if minimal}}
    COMMAND ${CMAKE_COMMAND} -E echo ""
    COMMAND ${CMAKE_COMMAND} -E echo "Minimal mode: Edit src/{{target_name}}.cpp to integrate with your code"
{{/if}}
    COMMENT "Displaying help"
)