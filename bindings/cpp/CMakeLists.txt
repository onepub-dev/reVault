cmake_minimum_required(VERSION 3.20)
project(revault_api LANGUAGES CXX)
find_package(Protobuf REQUIRED)
if(WIN32)
  find_library(REVAULT_NATIVE_LIBRARY NAMES revault_api.dll revault_api
    HINTS ${CMAKE_CURRENT_SOURCE_DIR}/../../rust/target/debug REQUIRED)
else()
  find_library(REVAULT_NATIVE_LIBRARY revault_api
    HINTS ${CMAKE_CURRENT_SOURCE_DIR}/../../rust/target/debug REQUIRED)
endif()
find_path(REVAULT_NATIVE_INCLUDE_DIR revault_api.h
  HINTS ${CMAKE_CURRENT_SOURCE_DIR}/../../rust/revault_bindings
  REQUIRED)
add_library(revault_api_cpp STATIC revault_api.cpp generated/revault_bindings.pb.cc)
target_compile_features(revault_api_cpp PUBLIC cxx_std_17)
target_include_directories(revault_api_cpp PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/generated>
  $<BUILD_INTERFACE:${REVAULT_NATIVE_INCLUDE_DIR}>
  $<INSTALL_INTERFACE:include>)
target_link_libraries(revault_api_cpp PUBLIC ${REVAULT_NATIVE_LIBRARY} protobuf::libprotobuf)
install(TARGETS revault_api_cpp ARCHIVE DESTINATION lib)
install(FILES revault_api.hpp generated/revault_bindings.pb.h DESTINATION include)
