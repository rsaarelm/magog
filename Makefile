RUSTPKG = RUST_PATH=$(PWD)/.rust:$(PWD)/lib/sdl2 rustpkg

all: shiny roamy crunchy

shiny:
	$(RUSTPKG) install shiny

roamy:
	$(RUSTPKG) install roamy

crunchy:
	$(RUSTPKG) install crunchy


clean:
	rm -rf bin/ build/ .rust/
	rm -rf lib/`uname -m`* # XXX: hacky.
