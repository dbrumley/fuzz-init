cmake_minimum_required(VERSION 3.10)
project(GPSParserExample C)

set(CMAKE_C_STANDARD 99)

# Compiler flags
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -g")
set(CMAKE_C_FLAGS_DEBUG "-O0 -fsanitize=address")
set(CMAKE_C_FLAGS_RELEASE "-O2")

# AddressSanitizer for debug builds
if(CMAKE_BUILD_TYPE STREQUAL "Debug" OR NOT CMAKE_BUILD_TYPE)
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fsanitize=address")
endif()

# Source files
set(GPS_LIB_SOURCES src/gps_parser.c)

# Create GPS parser library
add_library(gps_parser_lib STATIC ${GPS_LIB_SOURCES})
target_include_directories(gps_parser_lib PUBLIC src)

# Main GPS parser application
add_executable(gps_parser src/main.c)
target_link_libraries(gps_parser gps_parser_lib)

# Custom target to create test data
add_custom_target(test-data
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/test_data
    COMMAND ${CMAKE_COMMAND} -E echo "GPGGA,123456.78,1234,N,5678,W,1,08,0.9,545.4,M,46.9,M,,*47" > ${CMAKE_BINARY_DIR}/test_data/valid.nmea
    COMMAND ${CMAKE_COMMAND} -E echo "GPGGA,123456.78,1,N,0,W,1,08,0.9,545.4,M,46.9,M,,*47" > ${CMAKE_BINARY_DIR}/test_data/divide_by_zero.nmea
    COMMAND ${CMAKE_COMMAND} -E echo "GPGGA,123456.78,2,N,-79927771,W,1,08,0.9,545.4,M,46.9,M,,*47" > ${CMAKE_BINARY_DIR}/test_data/integer_overflow.nmea
    COMMAND ${CMAKE_COMMAND} -E echo "GPGGA,123456.78,3,N,-79927771,W,1,08,0.9,545.4,M,46.9,M,,*47" > ${CMAKE_BINARY_DIR}/test_data/oob_read.nmea
    COMMAND ${CMAKE_COMMAND} -E echo "Invalid GPS data without proper format" > ${CMAKE_BINARY_DIR}/test_data/invalid.nmea
    COMMENT "Creating test data files"
)

# Custom target to test the application
add_custom_target(test
    COMMAND $<TARGET_FILE:gps_parser> ${CMAKE_BINARY_DIR}/test_data/valid.nmea
    COMMAND $<TARGET_FILE:gps_parser> ${CMAKE_BINARY_DIR}/test_data/divide_by_zero.nmea
    COMMAND $<TARGET_FILE:gps_parser> ${CMAKE_BINARY_DIR}/test_data/integer_overflow.nmea
    DEPENDS gps_parser test-data
    COMMENT "Testing GPS parser with sample data"
)

# Install targets
install(TARGETS gps_parser DESTINATION bin)
install(FILES src/gps_parser.h DESTINATION include)