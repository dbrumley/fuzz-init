# fuzz/toolchains/standalone.cmake
set(FUZZER_TYPE "native" CACHE STRING "Active fuzzer type" FORCE)
set(CMAKE_BUILD_TYPE "Fuzzing" CACHE STRING "" FORCE)

# Use default C compiler or clang if available
find_program(CLANG clang)
if(CLANG AND NOT DEFINED CMAKE_C_COMPILER)
  set(CMAKE_C_COMPILER ${CLANG} CACHE STRING "" FORCE)
  set(CMAKE_CXX_COMPILER ${CLANG} CACHE STRING "" FORCE)
endif()

set(CMAKE_C_FLAGS_INIT   "-O1 -g -fno-omit-frame-pointer")
set(CMAKE_CXX_FLAGS_INIT "-O1 -g -fno-omit-frame-pointer")