cmake_minimum_required(VERSION 2.8)

if (NOT CMAKE_CROSSCOMPILING)
   # Render-font needs ImageMagick to emit pngs.
   find_package(ImageMagick REQUIRED)

   add_executable(render-font render-font.cpp)
   target_link_libraries(render-font ${M_LIBRARY})

   add_executable(emit-chardata emit-chardata.cpp)
   target_link_libraries(emit-chardata ${M_LIBRARY})

   add_executable(bake-data bake-data.cpp)

   EXPORT(TARGETS render-font emit-chardata bake-data FILE ${CMAKE_BINARY_DIR}/ImportExecutables.cmake )
endif ()

if (CMAKE_CROSSCOMPILING)
   SET(IMPORT_EXECUTABLES "${PROJECT_SOURCE_DIR}/build/ImportExecutables.cmake" CACHE FILEPATH "Point it to the export file from a native build")
   INCLUDE(${IMPORT_EXECUTABLES})
endif ()

function(bake bake_source bake_target)
   add_custom_command(
      OUTPUT ${CMAKE_BINARY_DIR}/${bake_target}
      COMMAND bake-data ${CMAKE_SOURCE_DIR}/${bake_source} ${CMAKE_BINARY_DIR}/${bake_target}
      DEPENDS ${bake_source} bake-data)
endfunction(bake)

