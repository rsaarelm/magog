cmake_minimum_required(VERSION 2.8)

project(xev)

include_directories(${PROJECT_SOURCE_DIR}/..)

# Black magic to make boost::range work with C++11 lambdas
add_definitions("-DBOOST_RESULT_OF_USE_DECLTYPE")

# XXX: Specify the warning flags in one place only.

# Be sure to specify Boost dir with "-isystem $BOOST_LOCATION" not "-I$BOOST_LOCATION", -Weffc++ goes
# crazy on Boost header code, but it wont be tripped by stuff from "-isystem" directories.
set(CMAKE_CXX_FLAGS "-std=c++0x -Wall -Werror -Woverloaded-virtual -Wold-style-cast ${CMAKE_CXX_FLAGS}")

add_executable(texbake
  tools/texbake.c
)

add_executable(fontbake
  tools/fontbake.c
)

add_executable(test_xev
  tests/*.cpp
)

# ENABLE_TESTING()
add_test(xev test_xev)

add_library(xev STATIC
  util.cpp
  contrib/stb/stb_image.c
  num.cpp
  hex.cpp
  shader.cpp
  color.cpp
  surface.cpp
  gldraw.cpp
  font.cpp
  imgui.cpp
  static_file.cpp
  transform.cpp
  geom_util.cpp
  wavefront.cpp
  game_loop.cpp
)

target_link_libraries(test_xev boost_unit_test_framework)

# "bake_file(picture.png)" adds a target picture.png.cpp which contains a
# static table that has the binary data from picture.png you can compile into
# your application.

# XXX: Presupposes data files are in "assets/" subdirectory.
# XXX: XEVPATH must be defined to point to the relative path to xev.
macro(bake_file file)
add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${file}.cpp
        COMMAND python ${CMAKE_SOURCE_DIR}/${XEVPATH}/xev/tools/databake.py ${CMAKE_CURRENT_SOURCE_DIR}/assets/${file} ${file} > ${CMAKE_CURRENT_BINARY_DIR}/${file}.cpp
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/assets/${file}
        )
endmacro(bake_file)
