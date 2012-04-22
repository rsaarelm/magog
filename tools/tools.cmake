if (NOT CMAKE_CROSSCOMPILING)
  add_executable(bake-data tools/bake-data.cpp)

  add_executable(build-atlas
    tools/build-atlas.cpp
    src/util/surface.cpp
    src/util/core.cpp
    src/util/format.cpp
    src/contrib/stb/stb_image.c
    )
  target_link_libraries(build-atlas ${M_LIBRARY})


  EXPORT(TARGETS bake-data build-atlas FILE ${CMAKE_BINARY_DIR}/ImportExecutables.cmake )
else ()
  # XXX: Assumes that the host build dir is called "build" and is in the same
  # parent directory with the cross-compilation build dir.

  if (NOT DEFINED BUILD_TOOLS_DIR)
    set(BUILD_TOOLS_DIR "${CMAKE_BINARY_DIR}/../build")
  endif ()

  set(IMPORT_EXECUTABLES "${BUILD_TOOLS_DIR}/ImportExecutables.cmake" CACHE FILEPATH "")
  include(${IMPORT_EXECUTABLES})
endif ()

add_custom_target(tools
  DEPENDS bake-data build-atlas)

function(bake bake_source bake_target)
  add_custom_command(
    OUTPUT ${CMAKE_BINARY_DIR}/${bake_target}
    COMMAND bake-data ${CMAKE_SOURCE_DIR}/${bake_source} ${CMAKE_BINARY_DIR}/${bake_target}
    DEPENDS ${bake_source} bake-data)
endfunction(bake)

macro(atlas data_target image_target)
  # TODO: Filter filenames from ARGN so they can be added to depends
  add_custom_command(
    OUTPUT ${CMAKE_BINARY_DIR}/${data_target} ${CMAKE_BINARY_DIR}/${image_target}.png
    COMMAND build-atlas ${CMAKE_BINARY_DIR}/${data_target} ${CMAKE_BINARY_DIR}/${image_target}.png ${ARGN}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    DEPENDS build-atlas)

  add_custom_command(
    OUTPUT ${CMAKE_BINARY_DIR}/${image_target}
    COMMAND bake-data ${CMAKE_BINARY_DIR}/${image_target}.png ${CMAKE_BINARY_DIR}/${image_target}
    DEPENDS ${image_target}.png bake-data)
endmacro(atlas)
