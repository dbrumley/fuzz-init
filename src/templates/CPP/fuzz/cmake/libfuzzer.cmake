# fuzz/toolchains/libfuzzer.cmake
set(FUZZER_TYPE "libfuzzer" CACHE STRING "Active fuzzer type" FORCE)
set(CMAKE_BUILD_TYPE "Fuzzing" CACHE STRING "" FORCE)

# Skip the compiler link test since libfuzzer provides main()
set(CMAKE_TRY_COMPILE_TARGET_TYPE "STATIC_LIBRARY")

find_program(CLANG clang)
find_program(CLANGXX clang++)
if(NOT CLANG OR NOT CLANGXX)
  message(FATAL_ERROR "clang/clang++ not found on PATH. Install LLVM/Clang.")
endif()
set(CMAKE_C_COMPILER   ${CLANG}   CACHE STRING "" FORCE)
set(CMAKE_CXX_COMPILER ${CLANGXX} CACHE STRING "" FORCE)

set(FUZZ_SAN_SET "address,undefined")

set(FUZZ_COMPILE_OPTS        "-fsanitize=fuzzer-no-link,${FUZZ_SAN_SET} -fno-omit-frame-pointer -g -O1")
set(FUZZ_LINK_OPTS_WITH_MAIN "-fsanitize=fuzzer,${FUZZ_SAN_SET}")
set(FUZZ_LINK_OPTS_NO_MAIN   "-fsanitize=${FUZZ_SAN_SET}")

# Apply compile opts globally
set(CMAKE_C_FLAGS_INIT   "${CMAKE_C_FLAGS_INIT} ${FUZZ_COMPILE_OPTS}")
set(CMAKE_CXX_FLAGS_INIT "${CMAKE_CXX_FLAGS_INIT} ${FUZZ_COMPILE_OPTS}")

# Helper interface targets. Avoid re-defining on re-include/try-compile
if(NOT DEFINED _FUZZ_LIBFUZZER_TOOLCHAIN_LOADED)
  set(_FUZZ_LIBFUZZER_TOOLCHAIN_LOADED TRUE)

  # create targets only if not already present
  if(NOT TARGET fuzz_link_with_main)
    add_library(fuzz_link_with_main INTERFACE)
    target_link_options(fuzz_link_with_main INTERFACE ${FUZZ_LINK_OPTS_WITH_MAIN})
  endif()

  if(NOT TARGET fuzz_link_no_main)
    add_library(fuzz_link_no_main INTERFACE)
    target_link_options(fuzz_link_no_main INTERFACE ${FUZZ_LINK_OPTS_NO_MAIN})
  endif()
endif()