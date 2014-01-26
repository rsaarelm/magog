CC ?= clang
AR ?= ar

RUSTPKG = RUST_PATH=$(PWD)/.rust:$(PWD)/lib/sdl2 rustpkg

LIBSDL2 := build/$(shell rustc --crate-file-name lib/sdl2/src/sdl2/lib.rs)
LIBGLFW := build/$(shell rustc --crate-file-name lib/glfw/src/lib.rs)
LIBCALX := build/$(shell rustc --crate-file-name src/calx/lib.rs)
LIBSTB := build/$(shell rustc --crate-file-name src/stb/lib.rs)

TARGET = shiny

bin/stb_demo: src/stb/stb_demo.rs $(LIBSTB)
	@mkdir -p bin/
	rustc -L build/ --out-dir bin/ $<

$(LIBCALX): src/calx/lib.rs
	@mkdir -p build/
	rustc --out-dir build/ --rlib $<

$(LIBGLFW): lib/glfw/src/lib.rs
	@mkdir -p build/
	rustc --out-dir build/ --rlib $<

$(LIBSDL2): lib/sdl2/src/sdl2/lib.rs
	@mkdir -p build/
	rustc --out-dir build/ --rlib $<

$(LIBSTB): src/stb/lib.rs build/libstb.a
	@mkdir -p build/
	rustc -L build/ --out-dir build/ $<

build/libstb.a:
	@mkdir -p build/
	$(CC) -fPIC -c -o build/stb_image.o src/stb/stb_image.c
	$(CC) -fPIC -c -o build/stb_truetype.o src/stb/stb_truetype.c
	$(AR) crs build/libstb.a build/stb_image.o build/stb_truetype.o

# all: shiny roamy crunchy
#
# # Call eg. "make run TARGET=shiny" to run different default binaries.
# run: $(TARGET)
# 	./bin/$(TARGET)
#
# %:
# 	$(RUSTPKG) install $@

clean:
	rm -rf bin/ build/

.PHONY: all clean
