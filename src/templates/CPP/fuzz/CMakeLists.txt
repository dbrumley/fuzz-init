cmake_minimum_required(VERSION 3.19)
project(HelloFuzzHarnesses LANGUAGES C CXX)

{{#if minimal}}
set(FUZZ_SRC_DIR "${CMAKE_SOURCE_DIR}/src" CACHE PATH "")
set(FUZZ_DRIVER_DIR "${CMAKE_SOURCE_DIR}/driver" CACHE PATH "")
{{else}}
set(FUZZ_SRC_DIR "${CMAKE_SOURCE_DIR}/fuzz/src" CACHE PATH "")
set(FUZZ_DRIVER_DIR "${CMAKE_SOURCE_DIR}/fuzz/driver" CACHE PATH "")
{{/if}}

set(FUZZ_HARNESS_SRCS 
  "${FUZZ_SRC_DIR}/{{project_name}}.cpp"
  # add more harness file sources here.
)

if(NOT FUZZ_HARNESS_SRCS)
  message(WARNING "No harness sources found in ${FUZZ_SRC_DIR}.")
endif()

foreach(harness ${FUZZ_HARNESS_SRCS})
  get_filename_component(stem "${harness}" NAME_WE)

  set(FUZZ_EXE ${stem}-${FUZZER_TYPE})

  set(sources ${harness})

  # libfuzzer does not require driver and links in its own main
  if(NOT FUZZER_TYPE STREQUAL "libfuzzer")
    list(APPEND sources ${FUZZ_DRIVER_DIR}/main.cpp)
  endif()

  # Create executable with determined sources
  add_executable(${FUZZ_EXE} ${sources})

  # Any project-level includes a harness may need
  {{#if minimal}}
  #target_include_directories(${FUZZ_EXE} PRIVATE ${CMAKE_SOURCE_DIR}/include)
  # Link against any project-level libraries. 
  # target_link_libraries(${FUZZ_EXE} PRIVATE mylib)  

  {{else}}
  # Use any project-level headers
  target_include_directories(${FUZZ_EXE} PRIVATE ${CMAKE_SOURCE_DIR}/include)
  # Link against any project-level libraries. 
  target_link_libraries(${FUZZ_EXE} PRIVATE mylib)  
  {{/if}}

  # Pull in libFuzzer's main only here via INTERFACE target if available
  if(TARGET fuzz_link_with_main)
    target_link_libraries(${FUZZ_EXE} PRIVATE fuzz_link_with_main)
  endif()  
  
  set_target_properties(${FUZZ_EXE} PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
  )


endforeach()

