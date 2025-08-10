# fuzz/toolchains/hongfuzz.cmake
set(FUZZER_TYPE "hongfuzz" CACHE STRING "Active fuzzer type" FORCE)
set(CMAKE_BUILD_TYPE "Fuzzing" CACHE STRING "" FORCE)

find_program(HF_CC hfuzz-clang)
find_program(HF_CXX hfuzz-clang++)
if(NOT HF_CC OR NOT HF_CXX)
  message(FATAL_ERROR "hongfuzz wrappers not found (hfuzz-clang/++). Install hongfuzz or skip this preset.")
endif()
set(CMAKE_C_COMPILER   ${HF_CC}  CACHE STRING "" FORCE)
set(CMAKE_CXX_COMPILER ${HF_CXX} CACHE STRING "" FORCE)
set(CMAKE_C_FLAGS_INIT   "-O1 -g -fno-omit-frame-pointer")
set(CMAKE_CXX_FLAGS_INIT "-O1 -g -fno-omit-frame-pointer")
