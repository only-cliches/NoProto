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
Macbook Air M1 with 8GB

```
====== SIZE BENCHMARK ======
NoProto:     size: 283b, zlib: 226b
Flatbuffers: size: 336b, zlib: 214b
PBuffers:    size: 220b, zlib: 163b
MessagePack: size: 431b, zlib: 245b
JSON:        size: 673b, zlib: 246b
BSON:        size: 600b, zlib: 279b

====== ENCODE BENCHMARK ======
NoProto:     2.645972s
Flatbuffers: 1.534927s
PBuffers:    2.175921s
MessagePack: 18.81802s
JSON:        3.48892s
BSON:        21.875655s

====== DECODE BENCHMARK ======
NoProto:     1.960938s
Flatbuffers: 104.1ms
PBuffers:    1.697763s
MessagePack: 10.457951s
JSON:        6.657481s
BSON:        21.234239s

====== DECODE ONE BENCHMARK ======
NoProto:     138.19ms
Flatbuffers: 8.563ms
PBuffers:    1.655446s
MessagePack: 9.588131s
JSON:        4.210371s
BSON:        20.255856s

====== UPDATE ONE BENCHMARK ======
NoProto:     181.864ms
Flatbuffers: 1.553352s
PBuffers:    3.914178s
MessagePack: 21.046205s
JSON:        5.158453s
BSON:        27.430653s
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