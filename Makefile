CC ?= clang
AR ?= ar

RUSTPKG = RUST_PATH=$(PWD)/.rust:$(PWD)/lib/sdl2 rustpkg

RUSTFLAGS = --opt-level 3 -L build/

LIBCALX := build/$(shell rustc --crate-file-name src/calx/lib.rs)
LIBSTB := build/$(shell rustc --crate-file-name src/stb/lib.rs)

LIBGLFW := build/$(shell rustc --crate-file-name lib/glfw-rs/src/lib.rs)
LIBGLES := build/$(shell rustc --crate-file-name lib/rust-opengles/lib.rs)
LIBPA := build/$(shell rustc --crate-file-name lib/portaudio-rs/src/portaudio/lib.rs)

TARGET = shiny

# Linux version.
GLFW_LINKARGS = --link-args "-lglfw3 -lGL -lX11 -lXxf86vm -lXrandr -lXi"
PA_LINKARGS = --link-args "-lasound -ljack"

# Build binary with "-Z lto" to make it smaller. Slows down build.

bin/shiny: src/shiny/main.rs $(LIBSTB) $(LIBGLFW) $(LIBCALX) $(LIBGLES) $(LIBPA)
	@mkdir -p bin/
	rustc $(RUSTFLAGS) $(GLFW_LINKARGS) $(PA_LINKARGS) -o $@ $<

$(LIBCALX): src/calx/lib.rs
	@mkdir -p build/
	rustc $(RUSTFLAGS) --out-dir build/ --rlib $<

$(LIBSTB): src/stb/lib.rs build/libstb.a
	@mkdir -p build/
	rustc $(RUSTFLAGS) --out-dir build/ $<

$(LIBGLFW): lib/glfw-rs/src/lib.rs build/libglfw3.a
	@mkdir -p build/
	rustc $(RUSTFLAGS) --out-dir build/ $<

$(LIBGLES): lib/rust-opengles/lib.rs
	@mkdir -p build/
	rustc $(RUSTFLAGS) --out-dir build/ --rlib $<

$(LIBPA): lib/portaudio-rs/src/portaudio/lib.rs build/libportaudio.a
	@mkdir -p build/
	rustc $(RUSTFLAGS) --out-dir build/ --rlib $<

build/libstb.a: cbuild/libstb.a
	@mkdir -p build/
	cp $< $@

cbuild/libstb.a: src/stb/stb_image.c src/stb/stb_truetype.h src/stb/stb_image_write.h
	@mkdir -p cbuild/
	$(CC) -fPIC -c -o cbuild/stb_image.o src/stb/stb_image.c
	$(CC) -fPIC -c -o cbuild/stb_truetype.o src/stb/stb_truetype.c
	$(CC) -fPIC -c -o cbuild/stb_image_write.o src/stb/stb_image_write.c
	$(AR) crs cbuild/libstb.a cbuild/stb_image.o cbuild/stb_truetype.o cbuild/stb_image_write.o

build/libglfw3.a: cbuild/glfw/src/libglfw3.a
	@mkdir -p build/
	cp $< $@

cbuild/glfw/src/libglfw3.a: lib/glfw/CMakeLists.txt
	@mkdir -p cbuild/glfw
	cd cbuild/glfw;\
	    cmake ../../lib/glfw;\
	    make;\
	    cp src/libglfw3.a ../../build/

build/libportaudio.a: cbuild/libportaudio.a
	@mkdir -p build/
	cp cbuild/libportaudio.a build/

cbuild/libportaudio.a: lib/portaudio/configure
	@mkdir -p cbuild
	cd lib/portaudio;\
	    ./configure CFLAGS=-fPIC;\
	    make
	cp lib/portaudio/lib/.libs/libportaudio.a cbuild/

clean:
	rm -rf bin/ build/

realclean: clean
	rm -rf cbuild/
	cd lib/portaudio; make distclean

.PHONY: all clean
