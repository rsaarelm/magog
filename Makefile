.PHONY: build/debug/telos build/release/telos run clean rundebug runrelease

build/debug/telos: build/debug/
	cd build/debug/; make telos

build/release/telos: build/release/
	cd build/release/; make telos

all: build/debug/telos build/release/telos

rundebug: build/debug/
	cd build/debug/; make telos && ./telos

runrelease: build/release/
	cd build/release/; make telos && ./telos

run: rundebug

#CMAKE_TYPE="-G MSYS Makefiles"
CMAKE_TYPE=

build/debug/:
	mkdir -p build/debug
	cd build/debug; \
	cmake $(CMAKE_TYPE) -D CMAKE_BUILD_TYPE=DEBUG ../..

build/release/:
	mkdir -p build/release
	cd build/release; \
	cmake $(CMAKE_TYPE) -D CMAKE_BUILD_TYPE=RELEASE ../..

clean:
	rm -rf build/