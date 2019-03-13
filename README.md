# Metal Info (`mtl-info`)
A command line tool for decoding and processing Apple's Metal Library files `metallib`.

## Installing
`mtl-info` is distributed with cargo. If Rust and Cargo are installed then follow the commands below to get it installed on your system.

```bash
cargo install mtl-info
# Check build output. If $PATH is setup correctly you can now run
mtl-info --help
```

## Introduction

An Apple Metal Library is a binary file format containing compiled Metal shaders. Many applications will include multiple such shaders which are all saved in the same Library file (usually called `default.metallib`). `mtl-info` helps parse and decode these files.

Each file begins with a CC char code to designate the file to macOS. The specific code is `MTLB`.

After that are a set of headers describing each shader within the file including it’s name and where it’s binary code exists within the library file.

Following that is the binary code data which is LLVM bitcode which can be converted using `llvm-dis` into a more human readable format.

## Usage
### Listing Entries
For listing the names of every Metal fragment or vertex shader in the Metal library.

```bash
mtl-info ./default.metallib list
```

### Processing Shader Code
Metal library files contain LLVM bitcode which can be disassembled into a more human-readable assembly format.

```bash
# Find entry by name
mtl-info ./default.metallib bitcode --with-name outlineRetina_frag

# Find entry by index
mtl-info ./default.metallib bitcode --with-index 3
```
