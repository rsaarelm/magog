#!/bin/sh

# Compile a static STB image library
cc -Wall -Os -DNDEBUG -c stb/stb_image.c stb/stb_image_write.c
ar crs libstb.a stb_image.o stb_image_write.o
rm stb_image.o stb_image_write.o
