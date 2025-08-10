cmake_minimum_required(VERSION 3.19)
project(MyApp LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(mylib STATIC
  src/foo.cpp
)
target_include_directories(mylib PUBLIC ${CMAKE_SOURCE_DIR}/include)

add_executable(myapp
  src/main.cpp
)
target_link_libraries(myapp PRIVATE mylib)

set_target_properties(myapp PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)

add_subdirectory(fuzz)

cmake_minimum_required(VERSION 3.16)
project(universal_fuzz CXX)

# ========= Basics =========
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include
                    ${CMAKE_CURRENT_SOURCE_DIR}/../include)

# ========= Tool / Env detection =========
find_program(CLANGXX   clang++)
find_program(AFLXX     afl-clang-fast++)
find_program(HFUZZXX   hfuzz-clang++)
# Project compiler info (used for standalone)
set(IS_PROJECT_CLANG OFF)
if (CMAKE_CXX_COMPILER_ID STREQUAL "Clang" OR CMAKE_CXX_COMPILER_ID STREQUAL "AppleClang")
  set(IS_PROJECT_CLANG ON)
endif()


set(COMMON_FLAGS "-g;-O1;-Wall;-Wextra;-fno-omit-frame-pointer;-fno-sanitize-recover=all")
set(SAN_FLAGS    "-fsanitize=address,undefined")
set(FUZZ_FLAG    "-fsanitize=fuzzer")

# Optional: detect one static lib above (skip fuzz/.git)
file(GLOB_RECURSE _LIBCAND
  "${CMAKE_CURRENT_SOURCE_DIR}/../lib*.a"
  "${CMAKE_CURRENT_SOURCE_DIR}/../*/lib*.a"
)
set(DETECTED_LIB "")
foreach(L ${_LIBCAND})
  string(FIND "${L}" "/fuzz/" _has_fuzz)
  string(FIND "${L}" "/.git/" _has_git)
  if (_has_fuzz EQUAL -1 AND _has_git EQUAL -1)
    set(DETECTED_LIB "${L}")
    break()
  endif()
endforeach()

# ========= Harness discovery =========
file(GLOB HARNESS_SRCS "${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp")
set(HARNESS_NAMES "")
foreach(SRC ${HARNESS_SRCS})
  get_filename_component(BASE "${SRC}" NAME_WE)
  list(APPEND HARNESS_NAMES "${BASE}")
endforeach()
list(SORT HARNESS_NAMES)

# ========= Configure-time summary =========
message(STATUS "=== Environment detection ===")
message(STATUS "Project compiler  : ${CMAKE_CXX_COMPILER_ID} (${CMAKE_CXX_COMPILER})")
message(STATUS "clang++ (for LF)  : ${CLANGXX}")
message(STATUS "afl-clang-fast++  : ${AFLXX}")
message(STATUS "hfuzz-clang++     : ${HFUZZXX}")
if (DETECTED_LIB)
  message(STATUS "External library  : ${DETECTED_LIB}")
else()
  message(STATUS "External library  : (none)")
endif()
message(STATUS "Harnesses         : ${HARNESS_NAMES}")

# Helper to apply common opts + link detected lib (if any)
function(apply_common_opts tgt)
  target_compile_options(${tgt} PRIVATE ${COMMON_FLAGS})
  if (DETECTED_LIB)
    target_link_libraries(${tgt} PRIVATE "${DETECTED_LIB}")
  endif()
endfunction()

# Collect for meta-targets/summary
set(ALL_TARGETS "")
set(LF_TARGETS "")
set(AFL_TARGETS "")
set(HFUZZ_TARGETS "")
set(STAND_TARGETS "")

foreach(H ${HARNESS_NAMES})
  set(HARNESS_SRC "${CMAKE_CURRENT_SOURCE_DIR}/src/${H}.cpp")
  set(DRIVER_SRC  "${CMAKE_CURRENT_SOURCE_DIR}/driver/main.cpp")

  # ---- standalone (always). If project compiler isn't Clang, link ASan+UBSan to satisfy sanitizer hooks in driver ----
  add_executable(${H}-standalone ${HARNESS_SRC} ${DRIVER_SRC})
  apply_common_opts(${H}-standalone)
  if (NOT IS_PROJECT_CLANG)
    target_compile_options(${H}-standalone PRIVATE ${SAN_FLAGS})
    target_link_options(${H}-standalone    PRIVATE ${SAN_FLAGS})
  endif()
  list(APPEND ALL_TARGETS ${H}-standalone)
  list(APPEND STAND_TARGETS ${H}-standalone)

  # ---- libFuzzer (if clang++ tool is found) ----
  if (CLANGXX)
    set(LF_OUT "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-libfuzzer")
    add_custom_command(
      OUTPUT  "${LF_OUT}"
      COMMAND ${CLANGXX}
              ${COMMON_FLAGS} ${SAN_FLAGS} ${FUZZ_FLAG} ${CMAKE_CXX_FLAGS}
              -I${CMAKE_CURRENT_SOURCE_DIR}/include -I${CMAKE_CURRENT_SOURCE_DIR}/../include
              ${HARNESS_SRC}
              $<$<BOOL:${DETECTED_LIB}>:${DETECTED_LIB}>
              -o "${LF_OUT}"
      DEPENDS ${HARNESS_SRC} ${DETECTED_LIB}
      COMMENT "[libFuzzer] building ${H}-libfuzzer with ASan+UBSan (+fuzzer)"
      VERBATIM
    )
    add_custom_target(${H}-libfuzzer ALL DEPENDS "${LF_OUT}")
    list(APPEND ALL_TARGETS ${H}-libfuzzer)
    list(APPEND LF_TARGETS  ${H}-libfuzzer)
  endif()

  # ---- AFL++ (if afl-clang-fast++ tool is found) ----
  if (AFLXX)
    set(AFL_OUT "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-afl")
    add_custom_command(
      OUTPUT  "${AFL_OUT}"
      COMMAND ${AFLXX}
              ${COMMON_FLAGS} ${SAN_FLAGS} ${CMAKE_CXX_FLAGS}
              -I${CMAKE_CURRENT_SOURCE_DIR}/include -I${CMAKE_CURRENT_SOURCE_DIR}/../include
              ${HARNESS_SRC} ${DRIVER_SRC}
              $<$<BOOL:${DETECTED_LIB}>:${DETECTED_LIB}>
              -o "${AFL_OUT}"
      DEPENDS ${HARNESS_SRC} ${DRIVER_SRC} ${DETECTED_LIB}
      COMMENT "[AFL++] building ${H}-afl with ASan+UBSan"
      VERBATIM
    )
    add_custom_target(${H}-afl ALL DEPENDS "${AFL_OUT}")
    list(APPEND ALL_TARGETS ${H}-afl)
    list(APPEND AFL_TARGETS ${H}-afl)
  endif()

  # ---- hongfuzz (if hfuzz-clang++ tool is found) ----
  if (HFUZZXX)
    set(HF_OUT "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-hongfuzz")
    add_custom_command(
      OUTPUT  "${HF_OUT}"
      COMMAND ${HFUZZXX}
              ${COMMON_FLAGS} ${SAN_FLAGS} ${CMAKE_CXX_FLAGS}
              -I${CMAKE_CURRENT_SOURCE_DIR}/include -I${CMAKE_CURRENT_SOURCE_DIR}/../include
              ${HARNESS_SRC} ${DRIVER_SRC}
              $<$<BOOL:${DETECTED_LIB}>:${DETECTED_LIB}>
              -o "${HF_OUT}"
      DEPENDS ${HARNESS_SRC} ${DRIVER_SRC} ${DETECTED_LIB}
      COMMENT "[hongfuzz] building ${H}-hongfuzz with ASan+UBSan"
      VERBATIM
    )
    add_custom_target(${H}-hongfuzz ALL DEPENDS "${HF_OUT}")
    list(APPEND ALL_TARGETS ${H}-hongfuzz)
    list(APPEND HFUZZ_TARGETS ${H}-hongfuzz)
  endif()
endforeach()

# Meta targets
add_custom_target(all_fuzzers     DEPENDS ${ALL_TARGETS})
add_custom_target(only_libfuzzer  DEPENDS ${LF_TARGETS})
add_custom_target(only_afl        DEPENDS ${AFL_TARGETS})
add_custom_target(only_hongfuzz   DEPENDS ${HFUZZ_TARGETS})
add_custom_target(only_standalone DEPENDS ${STAND_TARGETS})

# ---- Build-time recap ----
if (HARNESS_NAMES)
  set(SUMMARY_CMD "echo; echo '===== Fuzz targets summary =====';")
  foreach(H ${HARNESS_NAMES})
    string(APPEND SUMMARY_CMD
      "printf 'Harness: ${H}\\n';"
      "if [ -f '${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-libfuzzer' ]; then "
      "  echo '  - libfuzzer : built: ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-libfuzzer'; "
      "elif [ -z '${CLANGXX}' ]; then "
      "  echo '  - libfuzzer : skipped: install clang++ (apt: clang, brew: llvm) and rebuild'; "
      "else "
      "  echo '  - libfuzzer : skipped: check build log (LLVMFuzzerTestOneInput?)'; "
      "fi;"
      "if [ -f '${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-afl' ]; then "
      "  echo '  - afl       : built: ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-afl'; "
      "elif [ -z '${AFLXX}' ]; then "
      "  echo '  - afl       : skipped: install afl++ (apt: afl++, brew: afl-fuzz) and rebuild'; "
      "else "
      "  echo '  - afl       : skipped: check build log'; "
      "fi;"
      "if [ -f '${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-hongfuzz' ]; then "
      "  echo '  - hongfuzz  : built: ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-hongfuzz'; "
      "elif [ -z '${HFUZZXX}' ]; then "
      "  echo '  - hongfuzz  : skipped: install hongfuzz (apt: honggfuzz, brew: honggfuzz) and rebuild'; "
      "else "
      "  echo '  - hongfuzz  : skipped: check build log'; "
      "fi;"
      "if [ -f '${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-standalone' ]; then "
      "  echo '  - standalone: built: ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/${H}-standalone'; "
      "else "
      "  echo '  - standalone: skipped: (unexpected) see build log'; "
      "fi; echo;"
    )
  endforeach()

  add_custom_target(summary
    COMMAND ${CMAKE_COMMAND} -E env /bin/sh -c "${SUMMARY_CMD}"
    DEPENDS all_fuzzers
    VERBATIM
  )
endif()

# Tips
message(STATUS "Tips:")
message(STATUS "  - Configure (fresh):  cmake -S . -B build")
message(STATUS "  - Build all:          cmake --build build -j")
message(STATUS "  - Recap summary:      cmake --build build --target summary")
message(STATUS "  - Force clang toolchain globally (optional): CC=clang CXX=clang++ cmake -S . -B build")
