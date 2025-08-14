# fuzz/toolchains/afl.cmake
set(FUZZER_TYPE "afl" CACHE STRING "Active fuzzer type" FORCE)
set(CMAKE_BUILD_TYPE "Fuzzing" CACHE STRING "" FORCE)

find_program(AFL_CC afl-clang-fast)
if(NOT AFL_CC)
  message(FATAL_ERROR "AFL++ wrapper not found (afl-clang-fast). Install afl++ and ensure it is on PATH.")
endif()
set(CMAKE_C_COMPILER   ${AFL_CC}  CACHE STRING "" FORCE)
set(CMAKE_CXX_COMPILER ${AFL_CC}  CACHE STRING "" FORCE)
set(CMAKE_C_FLAGS_INIT   "-O1 -g -fno-omit-frame-pointer")
set(CMAKE_CXX_FLAGS_INIT "-O1 -g -fno-omit-frame-pointer")