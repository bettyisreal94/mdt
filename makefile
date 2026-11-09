all: mdt.exe

mdt.exe: src/main.rs
	cargo build
	cp target/debug/mdt mdt.exe

