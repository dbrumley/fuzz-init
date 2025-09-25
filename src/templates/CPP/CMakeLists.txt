cmake_minimum_required(VERSION 3.19)
project(HelloFuzz LANGUAGES C CXX)

# Put all outputs under the engine’s build dir in consistent subfolders.
# Works for single-config generators (Makefiles/Ninja).
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Multi-config generators (e.g., Visual Studio): set per-config too.
if(CMAKE_CONFIGURATION_TYPES)
  foreach(cfg ${CMAKE_CONFIGURATION_TYPES})
    string(TOUPPER "${cfg}" cfgU)
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_${cfgU} ${CMAKE_BINARY_DIR}/bin)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY_${cfgU} ${CMAKE_BINARY_DIR}/lib)
    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY_${cfgU} ${CMAKE_BINARY_DIR}/lib)
  endforeach()
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(mylib STATIC
  src/mylib.cpp
)
target_include_directories(mylib PUBLIC ${CMAKE_SOURCE_DIR}/include)

add_executable(hellofuzz
  src/main.cpp
)
target_link_libraries(hellofuzz PRIVATE mylib)

set_target_properties(hellofuzz PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
  EXCLUDE_FROM_ALL TRUE
)

add_subdirectory(fuzz)
