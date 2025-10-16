all: fizzbuzz_go fizzbuzz_c fizzbuzz_rs fizzbuzz_java

llvmir: fizzbuzz_go.ll fizzbuzz_c.ll fizzbuzz_rs.ll fizzbuzz_java.ll

fizzbuzz_go: fizzbuzz.go
	go build -o fizzbuzz_go fizzbuzz.go

fizzbuzz_c: fizzbuzz.c
	clang fizzbuzz.c -o fizzbuzz_c

fizzbuzz_rs: fizzbuzz.rs
	rustc fizzbuzz.rs -o fizzbuzz_rs

FizzBuzz.class: FizzBuzz.java
	javac FizzBuzz.java

# requires GraalVM with native-image component
#    gu install native-image
fizzbuzz_java: FizzBuzz.class
	javac FizzBuzz.java
	native-image --no-fallback -o fizzbuzz_java FizzBuzz

# requires tinygo (https://tinygo.org/)
fizzbuzz_go.ll: fizzbuzz.go
	tinygo build -o fizzbuzz_go.ll fizzbuzz.go

fizzbuzz_c.ll: fizzbuzz.c
	clang -emit-llvm -O0 -S fizzbuzz.c -o fizzbuzz_c.ll

fizzbuzz_rs.ll: fizzbuzz.rs
	rustc --emit=llvm-ir fizzbuzz.rs -o fizzbuzz_rs.ll

# requires GraalVM with llvm-toolchain component
#    gu install native-image
#    gu install llvm-toolchain
fizzbuzz_java.ll: FizzBuzz.class
	native-image -H:CompilerBackend=llvm --no-fallback -o fizzbuzz_java.ll FizzBuzz

# fizzbuzz_go.wasm: fizzbuzz.go
# 	tinygo build -o fizzbuzz_go.wasm -target=wasm fizzbuzz.go

# # requires wasi-sdk (https://github.com/WebAssembly/wasi-sdk)
# fizzbuzz_c.wasm: fizzbuzz.c
# 	clang -emit-llvm -O0 -S fizzbuzz.c -o fizzbuzz_c.wasm

# fizzbuzz_rs.wasm: fizzbuzz.rs
# 	rustc --emit=llvm-ir fizzbuzz.rs -o fizzbuzz_rs.wasm
