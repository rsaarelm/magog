TARGET=telos

.PHONY: tools build xbuild all run xrun clean

build: build/Makefile
	cd build/; make

build/Makefile: CMakeLists.txt
	mkdir -p build
	cd build/; cmake ..

# Tools are a separate target so that the cross-compile target can build them
# as local binaries.
tools: build/Makefile
	cd build/; make render-font && make emit-chardata && make bake-data

X_OPT=-D CMAKE_TOOLCHAIN_FILE=../cmake_scripts/Toolchain-mingw32.cmake

xbuild/Makefile: CMakeLists.txt tools
	mkdir -p xbuild
	cd xbuild/; cmake $(X_OPT) ..

xbuild: xbuild/Makefile
	cd xbuild/; make

all: build xbuild

run: build
	./build/telos

xrun: xbuild
	wine ./xbuild/telos

clean:
	rm -rf build/ xbuild/
