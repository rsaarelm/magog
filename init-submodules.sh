#!/bin/sh

git submodule update --init

# Build native GLFW.
mkdir -p .build/glfw
cd .build/glfw
cmake -DCMAKE_C_FLAGS="-fPIC -Os -DNDEBUG" ../../lib/glfw
make
cp src/libglfw3.a ../../libglfw.a
cd -

# Remove the temporary build directory to keep tup from complaining.
rm -rf .build/

cd lib/gl-rs
make lib
rm -rf examples/ lib/ bin/
rm deps/sax-rs/lib/*.rlib
rm deps/glfw-rs/src/lib/link.rs
cd -

cd lib/glfw-rs
make lib
rm -rf lib/
cd -
