use run_bench_json::JSONBench;
use run_bench_messagepack::MessagePackBench;

use crate::run_bench_protocol_buffers::ProtocolBufferBench;
use crate::run_bench_no_proto::NoProtoBench;
use crate::run_bench_flatbuffers::FlatBufferBench;
use crate::run_bench_bson::BSONBench;
use crate::run_bench_bincode::BincodeBench;

pub const LOOPS: usize = 1_0_000;

mod bench_fb;
mod bench_pb;
extern crate protobuf;
extern crate flatbuffers;
#[macro_use] 
extern crate json;
extern crate bson;
extern crate rmp;
extern crate serde;
extern crate bincode;

mod run_bench_no_proto;
mod run_bench_protocol_buffers;
mod run_bench_flatbuffers;
mod run_bench_messagepack;
mod run_bench_json;
mod run_bench_bson;
mod run_bench_bincode;

/*
1,000,000 iterations
0.4.2 - 144s
0.5.0 - 6s
*/

fn main() {

    println!("\n========= SIZE BENCHMARK =========");

    NoProtoBench::size_bench();
    FlatBufferBench::size_bench();
    BincodeBench::size_bench();
    ProtocolBufferBench::size_bench();
    MessagePackBench::size_bench();
    JSONBench::size_bench();
    BSONBench::size_bench();

    println!("\n======== ENCODE BENCHMARK ========");
    
    let base = NoProtoBench::encode_bench().unwrap();
    FlatBufferBench::encode_bench(base);
    BincodeBench::encode_bench(base);
    ProtocolBufferBench::encode_bench(base);
    MessagePackBench::encode_bench(base);
    JSONBench::encode_bench(base);
    BSONBench::encode_bench(base);

    println!("\n======== DECODE BENCHMARK ========");

    let base = NoProtoBench::decode_bench().unwrap();
    FlatBufferBench::decode_bench(base);
    BincodeBench::decode_bench(base);
    ProtocolBufferBench::decode_bench(base);
    MessagePackBench::decode_bench(base);
    JSONBench::decode_bench(base);
    BSONBench::decode_bench(base);

    println!("\n====== DECODE ONE BENCHMARK ======");

    let base = NoProtoBench::decode_one_bench().unwrap();
    FlatBufferBench::decode_one_bench(base);
    BincodeBench::decode_one_bench(base);
    ProtocolBufferBench::decode_one_bench(base);
    MessagePackBench::decode_one_bench(base);
    JSONBench::decode_one_bench(base);
    BSONBench::decode_one_bench(base);
    

    println!("\n====== UPDATE ONE BENCHMARK ======");

    let base = NoProtoBench::update_bench().unwrap();
    FlatBufferBench::update_bench(base);
    BincodeBench::update_bench(base);
    ProtocolBufferBench::update_bench(base);
    MessagePackBench::update_bench(base);
    JSONBench::update_bench(base);
    BSONBench::update_bench(base);
}

