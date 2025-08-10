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
target_link_libraries(hellofuzz PRIVATE mylib)

set_target_properties(hellofuzz PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

add_subdirectory(fuzz)

