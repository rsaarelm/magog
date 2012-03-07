.PHONY: debug release xdebug xrelease all run xrun clean

debug:
	mkdir -p debug
	cd debug/; cmake -D CMAKE_BUILD_TYPE=DEBUG .. && make

release:
	mkdir -p release
	cd release/; cmake -D CMAKE_BUILD_TYPE=RELEASE .. && make

X_OPT=-DCMAKE_TOOLCHAIN_FILE=../cmake_scripts/Toolchain-mingw32.cmake

xdebug:
	mkdir -p xdebug
	cd xdebug/; cmake $(X_OPT) -D CMAKE_BUILD_TYPE=DEBUG .. && make

xrelease:
	mkdir -p xrelease
	cd xrelease/; cmake $(X_OPT) -D CMAKE_BUILD_TYPE=RELEASE .. && make

all: debug release xdebug xrelease

run: debug
	./debug/telos

xrun: xdebug
	wine ./xdebug/telos

clean:
	rm -rf debug/ release/ xdebug/ xrelease/