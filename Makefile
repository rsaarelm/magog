all: build

build:
	cargo build

rebuild-dependencies:
	cargo build -u

run: build
	./target/magog

clean:
	rm -rf target/
