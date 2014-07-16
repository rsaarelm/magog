all: build

build:
	cargo build

rebuild-dependencies:
	cargo build -u

run: build
	./target/magog

doc: build
	rustdoc -L target/deps src/magog.rs
	rustdoc -L target/deps calx/src/lib.rs

clean:
	rm -rf target/
	rm -rf doc/
