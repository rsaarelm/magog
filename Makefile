all: build

build:
	cargo build

rebuild-dependencies:
	cargo build -u

run: build
	./target/calx

clean:
	rm -rf target/
