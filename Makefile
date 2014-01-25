RUSTPKG = RUST_PATH=$(PWD)/.rust:$(PWD)/lib/sdl2 rustpkg

TARGET=crunchy

all: shiny roamy crunchy

# Call eg. "make run TARGET=shiny" to run different default binaries.
run: $(TARGET)
	./bin/$(TARGET)

%:
	$(RUSTPKG) install $@

clean:
	rm -rf bin/ build/ .rust/
	rm -rf lib/`uname -m`* # XXX: hacky.

.PHONY: all clean
