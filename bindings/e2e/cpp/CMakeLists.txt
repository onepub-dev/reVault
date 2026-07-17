cmake_minimum_required(VERSION 3.20)
project(revault_cpp_conformance LANGUAGES CXX)

find_path(REVAULT_API_CPP_INCLUDE_DIR revault_api.hpp REQUIRED)
find_path(REVAULT_API_NATIVE_INCLUDE_DIR revault_api.h REQUIRED)
find_library(REVAULT_API_CPP_LIBRARY revault_api_cpp REQUIRED)
if(WIN32)
  find_library(REVAULT_API_NATIVE_LIBRARY NAMES revault_api.dll revault_api REQUIRED)
else()
  find_library(REVAULT_API_NATIVE_LIBRARY revault_api REQUIRED)
endif()
find_package(Protobuf REQUIRED)
add_executable(revault_cpp_conformance conformance.cpp)
target_compile_features(revault_cpp_conformance PRIVATE cxx_std_20)
target_include_directories(revault_cpp_conformance PRIVATE
  ${REVAULT_API_CPP_INCLUDE_DIR}
  ${REVAULT_API_CPP_INCLUDE_DIR}/generated
  ${REVAULT_API_NATIVE_INCLUDE_DIR})
target_link_libraries(revault_cpp_conformance PRIVATE
  ${REVAULT_API_CPP_LIBRARY}
  ${REVAULT_API_NATIVE_LIBRARY}
  protobuf::libprotobuf)
if(MSVC)
  target_compile_options(revault_cpp_conformance PRIVATE /W4 /WX)
else()
  target_compile_options(revault_cpp_conformance PRIVATE -Wall -Wextra -Wpedantic -Werror)
endif()
