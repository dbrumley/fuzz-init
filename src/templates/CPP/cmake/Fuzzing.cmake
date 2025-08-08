function(setup_fuzzing_flags engine out_var)
    set(flags_target fuzzing_flags_${engine})

    if(NOT TARGET ${flags_target})
        add_library(${flags_target} INTERFACE)

        if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
            if(engine STREQUAL "libfuzzer")
                target_compile_options(${flags_target} INTERFACE
                    -fsanitize=fuzzer,address -fno-omit-frame-pointer -g
                )
                target_link_options(${flags_target} INTERFACE
                    -fsanitize=fuzzer,address
                )
            elseif(engine STREQUAL "afl")
                target_compile_options(${flags_target} INTERFACE
                    -fsanitize=address -fno-omit-frame-pointer -g
                )
            elseif(engine STREQUAL "hongfuzz")
                target_compile_options(${flags_target} INTERFACE
                    -fsanitize=address -fno-omit-frame-pointer -g
                )
            elseif(engine STREQUAL "vanilla")
                # No instrumentation
            else()
                message(WARNING "Unknown fuzzing engine: ${engine}")
            endif()
        else()
            message(WARNING "Fuzzing instrumentation not supported with compiler: ${CMAKE_CXX_COMPILER_ID}. Building ${engine} target without instrumentation.")
        endif()
    endif()

    set(${out_var} ${flags_target} PARENT_SCOPE)
endfunction()

function(add_fuzz_target NAME ENGINE SRCS)
    setup_fuzzing_flags(${ENGINE} flags_target)

    add_library(${NAME}_${ENGINE}_obj OBJECT ${SRCS})
    target_link_libraries(${NAME}_${ENGINE}_obj PRIVATE ${flags_target})
    target_include_directories(${NAME}_${ENGINE}_obj PRIVATE ${CMAKE_SOURCE_DIR}/lib)

    add_executable(${NAME}_${ENGINE} $<TARGET_OBJECTS:${NAME}_${ENGINE}_obj>)
    target_link_libraries(${NAME}_${ENGINE} PRIVATE ${flags_target})
endfunction()

function(add_fuzz_targets_all)
    cmake_parse_arguments(FUZZ "" "NAME" "SRCS" ${ARGN})
    if(NOT FUZZ_NAME OR NOT FUZZ_SRCS)
        message(FATAL_ERROR "Usage: add_fuzz_targets_all(NAME name SRCS file.cpp [...])")
    endif()

    add_fuzz_target(${FUZZ_NAME} libfuzzer "${FUZZ_SRCS}")
    add_fuzz_target(${FUZZ_NAME} afl "${FUZZ_SRCS};fuzz/driver/main.cpp")
    add_fuzz_target(${FUZZ_NAME} hongfuzz "${FUZZ_SRCS}")
    add_fuzz_target(${FUZZ_NAME} vanilla "${FUZZ_SRCS};fuzz/driver/main.cpp")
endfunction()
