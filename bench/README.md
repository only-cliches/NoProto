# NoProto Benchmarks

The benchmarks in this folder are used to record performance progress and provide entirely subjective comparisons to other similar projects.

All libraries are working with an object that contains the same data and fields.  Data types are matched as much as possible.

### Size Benchmark
The example object is encoded once, and it's size in bytes is recorded as well as it's size in bytes with zlib compression.

### Encode Benchmark
The example object is encoded/serialized into the format supported by the various libraries.  Specifically, the benchmark measures how long it takes to get an owned `Vec<u8>` out of the library.

### Decode Benchmark
A single object is encoded, then the library decodes that object into it's parts 1,000,000 times.  Copying of the original buffer is only perfomed if it's needed by the library to complete decoding.  This measures how long it takes to get a shared immutable reference to all values in the object.

### Decode One Benchmark
A single object is encoded, then the library decodes a single property of the object 1,000,000 times.  Copying of the original buffer is only perfomed if it's needed by the library to complete decoding.  This measures how long it takes to get a shared immutable reference to a single value in the object.

### Update Benchmark
A single object is encoded, then the library should decode, update one property on the object then re encode the object 1,000,000 times.  The benchmark measures how long it takes to get from a deserialized buffer into another deserialized buffer with a single update performed in the new buffer.

Benchmarks can be ran with `cargo run --release`.

# Benchmarks Histry

## Dec 13, 2020
### v0.6.0
3.4Ghz i5 2017 21.5" iMac with 32 GB RAM

```
====== SIZE BENCHMARK ======
NoProto:     size: 283b, zlib: 226b  1x
Flatbuffers: size: 336b, zlib: 214b  1.2x
PBuffers:    size: 220b, zlib: 163b  0.8x
MessagePack: size: 431b, zlib: 245b  1.5x
JSON:        size: 673b, zlib: 246b  2.4x
BSON:        size: 600b, zlib: 279b  2.1x

====== ENCODE BENCHMARK ======
NoProto:     3.536623s   1.00x
Flatbuffers: 1.942583s   1.80x
PBuffers:    3.551301s   0.99x
MessagePack: 28.050727s  0.12x
JSON:        5.436352s   0.65x
BSON:        36.564978s  0.01x

====== DECODE BENCHMARK ======
NoProto:     2.496591s   1.00x
Flatbuffers: 320.065ms   8.00x
PBuffers:    2.888706s   0.80x
MessagePack: 16.576576s  0.15x
JSON:        8.957872s   0.30x
BSON:        32.770133s  0.08x

====== DECODE ONE BENCHMARK ======
NoProto:     206.966ms    1.00x
Flatbuffers: 13.127ms    16.00x
PBuffers:    2.715129s    0.07x
MessagePack: 14.300117s   0.01x
JSON:        7.836841s    0.02x
BSON:        37.513607s   0.01x

====== UPDATE ONE BENCHMARK ======
NoProto:     264.399ms     1.00x
Flatbuffers: 3.086538s     0.08x
PBuffers:    10.119442s     0.02x
MessagePack: 35.322739s    0.01x
JSON:        9.749246s     0.02x
BSON:        48.0097s    0.01x
```

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