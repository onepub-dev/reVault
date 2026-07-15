cmake_minimum_required(VERSION 3.20)
project(revault-api VERSION 0.1.0 LANGUAGES C CXX)
include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

set(REVAULT_API_LIBRARY "" CACHE FILEPATH "Path to the prebuilt revault-api library")
set(REVAULT_API_STATIC_LIBRARY "" CACHE FILEPATH "Path to the prebuilt static revault-api library")
if(NOT EXISTS "${REVAULT_API_LIBRARY}" OR NOT EXISTS "${REVAULT_API_STATIC_LIBRARY}")
  message(FATAL_ERROR "REVAULT_API_LIBRARY and REVAULT_API_STATIC_LIBRARY must name release artifacts")
endif()

install(FILES "${REVAULT_API_LIBRARY}" "${REVAULT_API_STATIC_LIBRARY}"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}")
install(FILES "${CMAKE_CURRENT_LIST_DIR}/../../../../../rust/revault_bindings/revault_api.h"
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
configure_file("${CMAKE_CURRENT_LIST_DIR}/../pkgconfig/revault-api.pc.in"
               "${CMAKE_CURRENT_BINARY_DIR}/revault-api.pc" @ONLY)
configure_package_config_file(
  "${CMAKE_CURRENT_LIST_DIR}/revault-api-config.cmake.in"
  "${CMAKE_CURRENT_BINARY_DIR}/revault-api-config.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/revault-api")
write_basic_package_version_file(
  "${CMAKE_CURRENT_BINARY_DIR}/revault-api-config-version.cmake"
  VERSION "${PROJECT_VERSION}" COMPATIBILITY SameMajorVersion)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/revault-api.pc"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig")
install(FILES
  "${CMAKE_CURRENT_BINARY_DIR}/revault-api-config.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/revault-api-config-version.cmake"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/revault-api")
