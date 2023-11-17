default: install

all: hooks install build

run:
	./bin/main

#run clang on the llvm ir to generate a binary 
build-ir:
	clang ./bin/main.ll -o ./bin/main

install-locally:
	cargo install --path=./cyclang

test: 
	cargo test -- --test-threads=1

test-release:
	cargo test --release -- --test-threads=1

test-parser:
	cargo test -- parser

clean:
	rm -rf ./bin/main*

h help:
	@grep '^[a-z]' Makefile

.PHONY: hooks
hooks:
	cd .git/hooks && ln -s -f ../../hooks/pre-push pre-push

install-mdbook:
	cargo install mdbook && cargo install mdbook-mermaid


s serve:
	cd book && mdbook serve


build-book:
	cd book && mdbook build

fib-wasm:
	cargo run -- --file=./examples/fib.cyc --target=wasm --emit-llvm-ir
	opt -O3 -S ./bin/main.ll -o ./bin/opt.ll
	llc -march=wasm32 -filetype=obj ./bin/main.ll -o ./bin/fib.o
	wasm-ld --no-entry --export-all -o ./bin/fib.wasm ./bin/fib.o
	llc -march=wasm32 -filetype=obj ./bin/opt.ll -o ./bin/fib-opt.o
	wasm-ld --no-entry --export-all -o ./bin/fib-opt.wasm ./bin/fib-opt.o
	cp ./bin/fib.wasm ./examples/wasm/fib.wasm
	cp ./bin/fib-opt.wasm ./examples/wasm/fib-opt.wasm
	node ./examples/wasm/fib.js

cargo-publish:
	cargo publish cyclang-macros && cargo publish cyclang