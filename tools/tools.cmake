if (NOT CMAKE_CROSSCOMPILING)
   # Render-font needs ImageMagick to emit pngs.
   find_package(ImageMagick REQUIRED)

   add_executable(render-font tools/render-font.cpp)
   target_link_libraries(render-font ${M_LIBRARY})

   add_executable(emit-chardata tools/emit-chardata.cpp)
   target_link_libraries(emit-chardata ${M_LIBRARY})

   add_executable(bake-data tools/bake-data.cpp)

   EXPORT(TARGETS render-font emit-chardata bake-data FILE ${CMAKE_BINARY_DIR}/ImportExecutables.cmake )
else ()
   # XXX: Assumes that the host build dir is called "build" and is in the same
   # parent directory with the cross-compilation build dir.

   if (NOT DEFINED BUILD_TOOLS_DIR)
     set(BUILD_TOOLS_DIR "${CMAKE_BINARY_DIR}/../build")
   endif ()

   set(IMPORT_EXECUTABLES "${BUILD_TOOLS_DIR}/ImportExecutables.cmake" CACHE FILEPATH "")
   include(${IMPORT_EXECUTABLES})
endif ()

function(bake bake_source bake_target)
   add_custom_command(
      OUTPUT ${CMAKE_BINARY_DIR}/${bake_target}
      COMMAND bake-data ${CMAKE_SOURCE_DIR}/${bake_source} ${CMAKE_BINARY_DIR}/${bake_target}
      DEPENDS ${bake_source} bake-data)
endfunction(bake)

function(bake_font font_file pixel_height first_char num_chars image_target chardata_target)
   # emit fontdata
   add_custom_command(
      OUTPUT ${CMAKE_BINARY_DIR}/${chardata_target}
      COMMAND emit-chardata ${pixel_height} ${first_char} ${num_chars}
         ${CMAKE_SOURCE_DIR}/${font_file} ${CMAKE_BINARY_DIR}/${chardata_target}
      DEPENDS ${font_file} emit-chardata)

   # generate intermediate .png
   add_custom_command(
      OUTPUT ${CMAKE_BINARY_DIR}/${image_target}.png
      COMMAND render-font ${pixel_height} ${first_char} ${num_chars}
         ${CMAKE_SOURCE_DIR}/${font_file} ${CMAKE_BINARY_DIR}/${image_target}.png
      DEPENDS ${font_file} render-font)

   # bake .png into image target
   add_custom_command(
      OUTPUT ${CMAKE_BINARY_DIR}/${image_target}
      COMMAND bake-data ${CMAKE_BINARY_DIR}/${image_target}.png ${CMAKE_BINARY_DIR}/${image_target}
      DEPENDS ${image_target}.png bake-data)
endfunction(bake_font)
