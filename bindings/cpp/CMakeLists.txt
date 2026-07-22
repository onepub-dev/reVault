cmake_minimum_required(VERSION 3.20)
project(revault_api LANGUAGES CXX)
find_package(flatbuffers 25.2.10 CONFIG QUIET)
if(NOT flatbuffers_FOUND)
  include(FetchContent)
  set(FLATBUFFERS_BUILD_TESTS OFF CACHE BOOL "" FORCE)
  set(FLATBUFFERS_BUILD_FLATC OFF CACHE BOOL "" FORCE)
  set(FLATBUFFERS_BUILD_FLATHASH OFF CACHE BOOL "" FORCE)
  FetchContent_Declare(flatbuffers
    GIT_REPOSITORY https://github.com/google/flatbuffers.git
    GIT_TAG v25.2.10
    GIT_SHALLOW TRUE)
  FetchContent_MakeAvailable(flatbuffers)
endif()
if(TARGET flatbuffers::flatbuffers)
  set(REVAULT_FLATBUFFERS_TARGET flatbuffers::flatbuffers)
elseif(TARGET flatbuffers)
  set(REVAULT_FLATBUFFERS_TARGET flatbuffers)
else()
  message(FATAL_ERROR "FlatBuffers CMake target was not created")
endif()
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
add_library(revault_api_cpp STATIC revault_api.cpp)
target_compile_features(revault_api_cpp PUBLIC cxx_std_20)
if(MSVC)
  target_compile_options(revault_api_cpp PRIVATE /W4 /WX)
else()
  target_compile_options(revault_api_cpp PRIVATE -Wall -Wextra -Wpedantic -Werror)
endif()
target_include_directories(revault_api_cpp PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
  $<BUILD_INTERFACE:${REVAULT_NATIVE_INCLUDE_DIR}>
  $<INSTALL_INTERFACE:include>)
target_include_directories(revault_api_cpp PRIVATE
  ${CMAKE_CURRENT_SOURCE_DIR}/generated/flatbuffers)
target_link_libraries(revault_api_cpp PUBLIC ${REVAULT_NATIVE_LIBRARY})
target_link_libraries(revault_api_cpp PRIVATE ${REVAULT_FLATBUFFERS_TARGET})
install(TARGETS revault_api_cpp ARCHIVE DESTINATION lib)
install(FILES revault_api.hpp domain_models.hpp DESTINATION include)
