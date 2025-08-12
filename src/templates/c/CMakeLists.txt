cmake_minimum_required(VERSION 3.19)
project({{project_name}} LANGUAGES C)

# Put all outputs under the engine's build dir in consistent subfolders.
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

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

add_library(lib STATIC
  src/lib.c
)
target_include_directories(lib PUBLIC ${CMAKE_SOURCE_DIR}/include)

add_executable({{project_name}}
  src/main.c
)
target_link_libraries({{project_name}} PRIVATE lib)

set_target_properties({{project_name}} PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

add_subdirectory(fuzz)