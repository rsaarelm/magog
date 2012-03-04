cmake_minimum_required(VERSION 2.8)

set(TARGETNAME telos)
project(${TARGETNAME})

# Settings from this file propagate to CMakeLists.txt files in subdirectories.

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake_scripts
                      ${CMAKE_MODULE_PATH})

# Black magic to make boost::range work with C++11 lambdas
add_definitions("-DBOOST_RESULT_OF_USE_DECLTYPE")



# Compiler-specific settings

if (CMAKE_COMPILER_IS_GNUCXX)
  # GCC-specific flags here
  set(CMAKE_CXX_FLAGS "-std=c++0x ${CMAKE_CXX_FLAGS}")
  set(CMAKE_CXX_FLAGS_DEBUG "-g -Wall -Werror -Woverloaded-virtual -Wold-style-cast")
  message(STATUS "Using GCC")
endif (CMAKE_COMPILER_IS_GNUCXX)

if (MSVC)
  # Definitions for MSVC compilers here.
  message(STATUS "Using MSVC")
endif (MSVC)


# OS-specific settings

if (WIN32)
  message(STATUS "On Windows")
endif (WIN32)

if (UNIX)
  message(STATUS "On *nix")
endif (UNIX)


# Package search

find_library(M_LIB m)

find_package(Boost COMPONENTS program_options REQUIRED)
find_package(OpenGL REQUIRED)
find_package(GLFW REQUIRED)
find_package(GLEW REQUIRED)
find_package(Portaudio)


# Helpful settings

set(CORELIBS ${M_LIB}
             ${Boost_PROGRAM_OPTIONS_LIBRARY}
             ${OPENGL_LIBRARY}
             ${GLFW_LIBRARY}
             ${GLEW_LIBRARY}
             ${PORTAUDIO_LIBRARIES})


# Project definition

include_directories("${PROJECT_SOURCE_DIR}/")

add_subdirectory(tools)

add_executable(${TARGETNAME}
  util/core.cpp
  contrib/stb/stb_image.c
  util/num.cpp
  util/hex.cpp
  util/shader.cpp
  util/color.cpp
  util/surface.cpp
  util/gldraw.cpp
  util/font.cpp
  util/imgui.cpp
  util/static_file.cpp
  util/transform.cpp
  util/geom_util.cpp
  util/wavefront.cpp
  util/game_loop.cpp

  ui/main.cpp
  ui/intro_screen.cpp
  ui/game_screen.cpp
  ui/message_buffer.cpp
  ui/config.cpp
  tiletex.cpp
  ttf.cpp
  world/actor.cpp
  world/world.cpp
  world/fov.cpp
  world/cavegen.cpp
)

target_link_libraries(${TARGETNAME} ${CORELIBS})

add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/ttf.cpp
        COMMAND tools/fontbake ${CMAKE_CURRENT_SOURCE_DIR}/assets/font.ttf 13 > ${CMAKE_CURRENT_BINARY_DIR}/ttf.cpp
        DEPENDS assets/font.ttf tools/fontbake
)

add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tiletex.cpp
        COMMAND tools/texbake ${CMAKE_CURRENT_SOURCE_DIR}/assets/tiles.png > ${CMAKE_CURRENT_BINARY_DIR}/tiletex.cpp
        DEPENDS assets/tiles.png tools/texbake
)
