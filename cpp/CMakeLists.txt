cmake_minimum_required(VERSION 3.14)
project(hexio_sdk VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Header-only library
add_library(hexio_sdk INTERFACE)
target_include_directories(hexio_sdk INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
    $<INSTALL_INTERFACE:include>
)

# Link platform-specific libraries
if(WIN32)
    target_link_libraries(hexio_sdk INTERFACE winhttp)
endif()

# Install target
install(TARGETS hexio_sdk EXPORT hexio_sdk-targets)
install(FILES hexio_sdk.h DESTINATION include)
install(EXPORT hexio_sdk-targets
    FILE hexio_sdk-config.cmake
    NAMESPACE hexio::
    DESTINATION lib/cmake/hexio_sdk
)

# Package config for find_package()
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/hexio_sdk-config-version.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)
install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/hexio_sdk-config-version.cmake"
    DESTINATION lib/cmake/hexio_sdk
)
