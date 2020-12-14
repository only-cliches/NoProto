# NoProto Benchmarks

The benchmarks in this folder are used to record performance progress and provide entirely subjective comparisons to other similar projects.

All libraries are working with an object that contains the same data and fields.  Data types are matched as much as possible.

### Size Benchmark
The example object is encoded once, and it's size in bytes is recorded as well as it's size in bytes with zlib compression.

### Encode Benchmark
The example object is encoded/serialized into the format supported by the various libraries.  Specifically, the benchmark measures how long it takes to get an owned `Vec<u8>` out of the library.

### Decode Benchmark
A single object is encoded, then the library decodes that object into it's parts 1,000,000 times.  Copying of the original buffer is only perfomed if it's needed by the library to complete decoding.  This measures how long it takes to get a shared immutable reference to all values in the object.

### Update Benchmark
A single object is encoded, then the library should decode, update then re encode the object 1,000,000 times.  The benchmark measures how long it takes to get from a deserialized buffer into another deserialized buffer with a single update performed in the new buffer.

Benchmarks can be ran with `cargo run --release`.

# Old Benchmarks

## Dec 13, 2020
### v0.6.0

====== SIZE BENCHMARK ======
NoProto:     size: 283b, zlib: 226b
Flatbuffers: size: 336b, zlib: 214b
PBuffers:    size: 220b, zlib: 163b
MessagePack: size: 431b, zlib: 245b
JSON:        size: 673b, zlib: 246b

====== ENCODE BENCHMARK ======
NoProto:     3.645993s
Flatbuffers: 2.031044s
PBuffers:    3.814115s
MessagePack: 29.39988s
JSON:        5.54222s

====== DECODE BENCHMARK ======
NoProto:     2.625244s
Flatbuffers: 156.943ms
PBuffers:    2.864848s
MessagePack: 16.351103s
JSON:        9.231317s

====== DECODE ONE BENCHMARK ======
NoProto:     204.795ms
Flatbuffers: 11.63ms
PBuffers:    2.828876s
MessagePack: 15.166845s
JSON:        7.23197s

====== UPDATE BENCHMARK ======
NoProto:     267.654ms
Flatbuffers: 2.214119s
PBuffers:    6.673877s
MessagePack: 32.867811s
JSON:        8.96524s

## Dec 1, 2020
### v0.5.1 
Macbook Air M1 with 8GB

```
====== SIZE BENCHMARK ======
NoProto:     size: 408b, zlib: 321b
Flatbuffers: size: 336b, zlib: 214b
PBuffers:    size: 220b, zlib: 163b

====== ENCODE BENCHMARK ======
NoProto:     5.707984s
Flatbuffers: 1.556862s
PBuffers:    2.209196s

====== DECODE BENCHMARK ======
NoProto:     9.161315s
Flatbuffers: 105.914ms
PBuffers:    1.691681s

====== UPDATE BENCHMARK ======
NoProto:     602.446ms
Flatbuffers: 1.512228s
PBuffers:    3.791677s
```