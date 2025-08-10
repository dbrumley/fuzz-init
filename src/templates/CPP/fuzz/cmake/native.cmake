# fuzz/toolchains/native.cmake
set(FUZZER_TYPE "native" CACHE STRING "Active fuzzer type" FORCE)
set(CMAKE_BUILD_TYPE "Fuzzing" CACHE STRING "" FORCE)

set(CMAKE_C_FLAGS_INIT   "-O2 -g -fno-omit-frame-pointer")
set(CMAKE_CXX_FLAGS_INIT "-O2 -g -fno-omit-frame-pointer")
