use crate::run_bench_protocol_buffers::ProtocolBufferBench;
use crate::run_bench_no_proto::NoProtoBench;
use crate::run_bench_flatbuffers::FlatBufferBench;

pub const LOOPS: usize = 1_000_000;

mod bench_fb;
mod bench_pb;
extern crate protobuf;
extern crate flatbuffers;
#[macro_use] 
extern crate json;
extern crate perf_stats;

mod run_bench_no_proto;
mod run_bench_protocol_buffers;
mod run_bench_flatbuffers;

/*
1,000,000 iterations
0.4.2 - 144s
0.5.0 - 6s

*/

fn main() {

    println!("====== SIZE BENCHMARK ======");

    NoProtoBench::size_bench();
    FlatBufferBench::size_bench();
    ProtocolBufferBench::size_bench();

    println!("\n====== ENCODE BENCHMARK ======");
    
    NoProtoBench::encode_bench().unwrap();
    FlatBufferBench::encode_bench();
    ProtocolBufferBench::encode_bench();

/*
    println!("\n====== DECODE BENCHMARK ======");
    
    NoProtoBench::decode_bench().unwrap();
    FlatBufferBench::decode_bench();
    ProtocolBufferBench::decode_bench();

    println!("\n====== UPDATE BENCHMARK ======");

    NoProtoBench::update_bench().unwrap();
    FlatBufferBench::update_bench();
    ProtocolBufferBench::update_bench();
    */
}

