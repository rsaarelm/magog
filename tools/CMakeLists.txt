cmake_minimum_required(VERSION 2.8)

add_executable(fontbake fontbake.c)
target_link_libraries(fontbake ${M_LIB})

add_executable(texbake texbake.c)
target_link_libraries(texbake ${M_LIB})

