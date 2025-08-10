cmake_minimum_required(VERSION 3.19)
project(HelloFuzz LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(mylib STATIC
  src/mylib.cpp
)
target_include_directories(mylib PUBLIC ${CMAKE_SOURCE_DIR}/include)

add_executable(hellofuzz
  src/main.cpp
)
target_link_libraries(myapp PRIVATE mylib)

set_target_properties(myapp PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

add_subdirectory(fuzz)


cmake_minimum_required(VERSION 3.20)
project({{project_name}} 
        DESCRIPTION "fuzz-init generated project"
        LANGUAGES CXX)


# ========= Basics =========
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})

# Library source files
set(LIB_SOURCES src/mylib.cpp)

# Create a library with everything but main. 
# This way testing and fuzzing harnesses can cleanly link 
add_library({{project_name}}_lib STATIC ${LIB_SOURCES})
target_include_directories({{project_name}}_lib PUBLIC include)
target_compile_features({{project_name}}_lib PUBLIC cxx_std_11)

# Create main executable
add_executable({{project_name}} src/main.cpp)
target_link_libraries({{project_name}} PUBLIC {{project_name}}_lib)


# --- Fuzzing --- 

# Include the Fuzzing cmake extension 
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(Fuzzing)


# Find all fuzz harnesses under fuzz/src
file(GLOB FUZZ_HARNESSES CONFIGURE_DEPENDS "${CMAKE_SOURCE_DIR}/fuzz/src/*.cpp")

# Add all fuzzing harnesses
foreach(harness ${FUZZ_HARNESSES})
    get_filename_component(harness_name ${harness} NAME_WE)
    add_fuzz_targets_all(NAME ${harness_name} LIBS mylib SRCS ${harness})
    #target_link_libraries(${harness_name}_libfuzzer PRIVATE mylib)
    #target_link_libraries(${harness_name}_afl PRIVATE mylib)
    #target_link_libraries(${harness_name}_hongfuzz PRIVATE mylib)
    #target_link_libraries(${harness_name}_vanilla PRIVATE mylib)
endforeach()

# Add test subdirectory (unit tests)
#add_subdirectory(test)

# Override default test target to run unit tests
#add_custom_target(test
#    COMMAND ${CMAKE_COMMAND} --build . --target run_tests
#    DEPENDS test_{{project_name}}
#    COMMENT "Running unit tests"
#)

# Fuzz target - builds the default fuzzer
#add_custom_target(fuzz
#    COMMAND ${CMAKE_COMMAND} --build fuzz 
#    COMMENT "Building {{default_fuzzer}} fuzzer"
#)


# Subdirectory for fuzz targets
#add_subdirectory(fuzz)

# Install targets
install(TARGETS {{target_name}} DESTINATION bin)
