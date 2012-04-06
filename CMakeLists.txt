cmake_minimum_required(VERSION 2.8)

set(TARGETNAME magog)
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


# Package search

find_library(M_LIBRARY m)

find_package(Boost)
find_package(OpenGL REQUIRED)
find_package(SDL REQUIRED)

if (CROSS_COMPILING_TO_WINDOWS)
  # Link SDL statically
  execute_process(COMMAND ${TARGET_SDL_CONFIG} --static-libs OUTPUT_VARIABLE SDL_LIBRARY OUTPUT_STRIP_TRAILING_WHITESPACE)
  set(SDL_LIBRARY "-Wl,-Bstatic ${SDL_LIBRARY} -Wl,-Bdynamic")

  # Link standard C and C++ libraries statically
  set(CMAKE_SHARED_LIBRARY_LINK_CXX_FLAGS "-static-libgcc -static-libstdc++")
endif (CROSS_COMPILING_TO_WINDOWS)


# Helpful settings

set(CORELIBS
  ${SDL_LIBRARY}
  ${OPENGL_LIBRARY}
  ${M_LIBRARY}
)


# Project definition

include_directories("${PROJECT_SOURCE_DIR}/" "${PROJECT_BINARY_DIR}/")

add_executable(${TARGETNAME}
  util/core.cpp
  contrib/stb/stb_image.c
  util/num.cpp
  util/hex.cpp
  util/color.cpp
  util/surface.cpp
  util/gldraw.cpp
  util/gl_texture.cpp
  util/fonter_system.cpp
  util/format.cpp
  util/imgui_system.cpp
  util/transform.cpp
  util/geom_util.cpp
  util/wavefront.cpp
  util/game_loop.cpp
  util/audio.cpp
  util/mixer.cpp
  util/sdl_util.cpp

  ui/main.cpp
  ui/intro_screen.cpp
  ui/display_system.cpp
  ui/ui_fx_system.cpp
  ui/game_screen.cpp
  ui/hud_system.cpp
  ui/tile_drawable.cpp
  ui/sprite_system.cpp
  ui/registry.cpp

  world/entities_system.cpp
  world/terrain_system.cpp
  world/spatial_system.cpp
  world/fov_system.cpp
  world/factory_system.cpp
  world/action_system.cpp
  world/cycler_system.cpp
  world/footprint.cpp
  world/location.cpp
  world/cavegen.cpp

  font_image.hpp
  font_data.hpp

  tile_rect.hpp
  tile_atlas.hpp

  buildname.hpp
)

target_link_libraries(${TARGETNAME} ${CORELIBS})

include(tools/tools.cmake)

add_custom_target(run
  COMMAND ${CMAKE_CURRENT_BINARY_DIR}/${TARGETNAME}
  DEPENDS ${TARGETNAME}
)

bake_font(assets/pf_tempesta_seven_extended_bold.ttf 13 32 96 font_image.hpp font_data.hpp)

atlas(tile_rect.hpp tile_atlas.hpp
  -n 8 assets/tiles/000-8-terrain.png
  -n 6 assets/tiles/008-6-slope.png
  -n 8 assets/tiles/014-8-wall.png
  -n 16 assets/tiles/022-16-creatures.png
  -n 15 assets/tiles/038-15-anim-creatures.png
)

# Build identifier string

add_custom_command(
  OUTPUT buildname.hpp
  COMMAND git log --pretty=format:\\"%h\\" -1 > ${CMAKE_CURRENT_BINARY_DIR}/buildname.hpp
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR})

# Documentation generation

find_package(Doxygen)
if(DOXYGEN_FOUND)
  configure_file(${CMAKE_CURRENT_SOURCE_DIR}/Doxyfile.in
    ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile @ONLY)
  add_custom_target(doc
    ${DOXYGEN_EXECUTABLE} ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "Generating API documentation with Doxygen" VERBATIM)
endif(DOXYGEN_FOUND)
