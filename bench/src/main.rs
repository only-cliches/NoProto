use run_bench_json::JSONBench;
use run_bench_messagepack::MessagePackBench;

use crate::run_bench_protocol_buffers::ProtocolBufferBench;
use crate::run_bench_no_proto::NoProtoBench;
use crate::run_bench_flatbuffers::FlatBufferBench;
use crate::run_bench_bson::BSONBench;

pub const LOOPS: usize = 1_000_000;

mod bench_fb;
mod bench_pb;
extern crate protobuf;
extern crate flatbuffers;
#[macro_use] 
extern crate json;
extern crate bson;

mod run_bench_no_proto;
mod run_bench_protocol_buffers;
mod run_bench_flatbuffers;
mod run_bench_messagepack;
mod run_bench_json;
mod run_bench_bson;

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
    MessagePackBench::size_bench();
    JSONBench::size_bench();
    BSONBench::size_bench();

    println!("\n====== ENCODE BENCHMARK ======");
    
    NoProtoBench::encode_bench().unwrap();
    FlatBufferBench::encode_bench();
    ProtocolBufferBench::encode_bench();
    MessagePackBench::encode_bench();
    JSONBench::encode_bench();
    BSONBench::encode_bench();

    println!("\n====== DECODE BENCHMARK ======");
    
    NoProtoBench::decode_bench().unwrap();
    FlatBufferBench::decode_bench();
    ProtocolBufferBench::decode_bench();
    MessagePackBench::decode_bench();
    JSONBench::decode_bench();
    BSONBench::decode_bench();

    println!("\n====== DECODE ONE BENCHMARK ======");
    
    NoProtoBench::decode_one_bench().unwrap();
    FlatBufferBench::decode_one_bench();
    ProtocolBufferBench::decode_one_bench();
    MessagePackBench::decode_one_bench();
    JSONBench::decode_one_bench();
    BSONBench::decode_one_bench();

    println!("\n====== UPDATE ONE BENCHMARK ======");

    NoProtoBench::update_bench().unwrap();
    FlatBufferBench::update_bench();
    ProtocolBufferBench::update_bench();
    MessagePackBench::update_bench();
    JSONBench::update_bench();
    BSONBench::update_bench();
}

