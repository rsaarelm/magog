cmake_minimum_required(VERSION 2.8)

set(TARGETNAME telos)

add_subdirectory(xev)

include_directories("${PROJECT_SOURCE_DIR}/")
set(CMAKE_SHARED_LIBRARY_LINK_CXX_FLAGS "-static-libgcc -static-libstdc++")

set(CMAKE_CXX_FLAGS "-std=c++0x -Wall -Woverloaded-virtual -Wold-style-cast ${CMAKE_CXX_FLAGS}")

# XXX: Hardcoded directory.
set(CMAKE_CXX_FLAGS "-I/c/Users/winuser/work/telos/ ${CMAKE_CXX_FLAGS}")
add_definitions("-DGLEW_STATIC")

# Black magic to make boost::range work with C++11 lambdas
add_definitions("-DBOOST_RESULT_OF_USE_DECLTYPE")

find_package(OpenGL REQUIRED)

project(${TARGETNAME})

add_executable(${TARGETNAME}
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

set_target_properties(${TARGETNAME} PROPERTIES LINK_FLAGS -static)

add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/ttf.cpp
        COMMAND xev/fontbake ${CMAKE_CURRENT_SOURCE_DIR}/assets/font.ttf 13 > ${CMAKE_CURRENT_BINARY_DIR}/ttf.cpp
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/assets/font.ttf fontbake
)

add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tiletex.cpp
        COMMAND xev/texbake ${CMAKE_CURRENT_SOURCE_DIR}/assets/tiles.png > ${CMAKE_CURRENT_BINARY_DIR}/tiletex.cpp
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/assets/tiles.png texbake
)

target_link_libraries(${TARGETNAME} xev)
target_link_libraries(${TARGETNAME} "-lglfw")
target_link_libraries(${TARGETNAME} "-lboost_program_options")

if(${CMAKE_SYSTEM_NAME} MATCHES "Windows")
target_link_libraries(${TARGETNAME} "-lglew32 -lglu32 -lopengl32")
else()
target_link_libraries(${TARGETNAME} "-lglew -lglu -lgl")
endif()

get_property(targetpath TARGET ${TARGETNAME} PROPERTY LOCATION)

# Optional step, minimize executable size.
if(${CMAKE_BUILD_TYPE} MATCHES RELEASE)
message(STATUS "Building release")

if(${CMAKE_SYSTEM_NAME} MATCHES "Windows")
set(CMAKE_CXX_FLAGS "-mwindows ${CMAKE_CXX_FLAGS}")
endif(${CMAKE_SYSTEM_NAME} MATCHES "Windows")

add_custom_command(
        TARGET ${TARGETNAME} POST_BUILD
        COMMAND strip ${targetpath}
        COMMAND upx ${targetpath})
endif(${CMAKE_BUILD_TYPE} MATCHES RELEASE)
