# fuzz/toolchains/honggfuzz.cmake
set(FUZZER_TYPE "honggfuzz" CACHE STRING "Active fuzzer type" FORCE)
set(CMAKE_BUILD_TYPE "Fuzzing" CACHE STRING "" FORCE)

find_program(HFUZZ_CC hfuzz-clang)
if(NOT HFUZZ_CC)
  message(FATAL_ERROR "HonggFuzz wrapper not found (hfuzz-clang). Install honggfuzz and ensure it is on PATH.")
endif()
set(CMAKE_C_COMPILER   ${HFUZZ_CC} CACHE STRING "" FORCE)
set(CMAKE_CXX_COMPILER ${HFUZZ_CC} CACHE STRING "" FORCE)
set(CMAKE_C_FLAGS_INIT   "-O1 -g -fno-omit-frame-pointer")
set(CMAKE_CXX_FLAGS_INIT "-O1 -g -fno-omit-frame-pointer")