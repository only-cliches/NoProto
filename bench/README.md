# NoProto Benchmarks

The benchmarks in this folder are used to record performance progress and provide entirely subjective comparisons to other similar projects.

All libraries are working with an object that contains the same data and fields.  Data types are matched as much as possible.

### Size Benchmark
The example object is encoded once, and it's size in bytes is recorded as well as it's size in bytes with zlib compression.

### Encode Benchmark
The example object is encoded/serialized into the format supported by the various libraries.  Specifically, the benchmark measures how long it takes to get an owned `Vec<u8>` out of the library.

### Decode Benchmark
A single object is encoded, then the library decodes that object into it's parts 1,000,000 times.  Copying of the original buffer is only perfomed if it's needed by the library to complete decoding.  This measures how long it takes to go from a `Vec<u8>` to a shared immutable reference to all properties/values in the object.

### Decode One Benchmark
A single object is encoded, then the library decodes a single property of that object 1,000,000 times.  Copying of the original buffer is only perfomed if it's needed by the library to complete decoding.  This measures how long it takes to go from a `Vec<u8>` to a shared immutable reference of a single value in the object.

### Update Benchmark
A single object is encoded, then the library should decode, update one property on the object then re encode the object 1,000,000 times.  The benchmark measures how long it takes to get from a deserialized buffer into another deserialized buffer with a single update performed in the new buffer.

Benchmarks can be ran with `cargo run --release`.

# Benchmarks Histry

## Dec 15, 2020
### v0.6.1
3.4Ghz i5 2017 21.5" iMac with 32 GB RAM

```
====== SIZE BENCHMARK ======
NoProto:     size: 284b, zlib: 229b
Flatbuffers: size: 336b, zlib: 214b
PBuffers:    size: 220b, zlib: 163b
MessagePack: size: 431b, zlib: 245b
JSON:        size: 673b, zlib: 246b
BSON:        size: 600b, zlib: 279b

====== ENCODE BENCHMARK ======
NoProto:      3636ms 1.00
Flatbuffers:  1884ms 1.93
PBuffers:     3522ms 1.03
MessagePack: 29052ms 0.13
JSON:         5319ms 0.68
BSON:        39693ms 0.09

====== DECODE BENCHMARK ======
NoProto:      2664ms 1.00
Flatbuffers:   150ms 17.66
PBuffers:     3316ms 0.80
MessagePack: 16770ms 0.16
JSON:         9237ms 0.29
BSON:        36965ms 0.07

====== DECODE ONE BENCHMARK ======
NoProto:       200ms 1.00
Flatbuffers:    12ms 16.39
PBuffers:     2794ms 0.07
MessagePack: 16332ms 0.01
JSON:         7015ms 0.03
BSON:        34030ms 0.01

====== UPDATE ONE BENCHMARK ======
NoProto:       406ms 1.00
Flatbuffers:  2575ms 0.16
PBuffers:     7470ms 0.05
MessagePack: 33735ms 0.01
JSON:         8743ms 0.05
BSON:        54590ms 0.01
```

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
NoProto:     3.536623s   
Flatbuffers: 1.942583s   
PBuffers:    3.551301s   
MessagePack: 28.050727s  
JSON:        5.436352s   
BSON:        36.564978s  

====== DECODE BENCHMARK ======
NoProto:     2.496591s   
Flatbuffers: 320.065ms  
PBuffers:    2.888706s   
MessagePack: 16.576576s  
JSON:        8.957872s  
BSON:        32.770133s  

====== DECODE ONE BENCHMARK ======
NoProto:     206.966ms    
Flatbuffers: 13.127ms    
PBuffers:    2.715129s    
MessagePack: 14.300117s   
JSON:        7.836841s    
BSON:        37.513607s   

====== UPDATE ONE BENCHMARK ======
NoProto:     264.399ms    
Flatbuffers: 3.086538s     
PBuffers:    10.119442s     
MessagePack: 35.322739s    
JSON:        9.749246s   
BSON:        48.0097s    
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