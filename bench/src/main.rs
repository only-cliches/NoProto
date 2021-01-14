use run_bench_json::JSONBench;
use run_bench_messagepack::MessagePackBench;

use crate::run_bench_protocol_buffers::ProtocolBufferBench;
use crate::run_bench_no_proto::NoProtoBench;
use crate::run_bench_flatbuffers::FlatBufferBench;
use crate::run_bench_bson::BSONBench;
use crate::run_bench_bincode::BincodeBench;
use crate::run_bench_avro::AvroBench;
use crate::run_bench_prost::ProstBench;
use crate::run_bench_flexbuffers::FlexBench;
// use crate::run_bench_rawbson::RawBSONBench;
use crate::run_bench_abomonation::AbomBench;
pub const LOOPS: usize = 1_000_000;

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
#[macro_use] 
extern crate abomonation;


mod run_bench_no_proto;
mod run_bench_protocol_buffers;
mod run_bench_flatbuffers;
mod run_bench_messagepack;
mod run_bench_json;
mod run_bench_bson;
mod run_bench_bincode;
mod run_bench_prost;
mod run_bench_avro;
mod run_bench_flexbuffers;
mod run_bench_rawbson;
mod run_bench_abomonation;

/*
1,000,000 iterations
0.4.2 - 144s
0.5.0 - 6s
*/

fn main() {


    NoProtoBench::setup_bench();
    AvroBench::setup_bench();
    FlatBufferBench::setup_bench();

    println!("\n========= SIZE BENCHMARK =========");

    let np_size = NoProtoBench::size_bench();
    let fb_size = FlatBufferBench::size_bench();
    let bn_size = BincodeBench::size_bench();
    let pb_size = ProtocolBufferBench::size_bench();
    let msg_size = MessagePackBench::size_bench();
    let json_size = JSONBench::size_bench();
    let bson_size = BSONBench::size_bench();
    let pro_size = ProstBench::size_bench();
    let avro_size = AvroBench::size_bench();
    let flx_size = FlexBench::size_bench();
    let abo_size = AbomBench::size_bench();
    // let rbso_size = RawBSONBench::size_bench();

    println!("\n======== ENCODE BENCHMARK ========");
    
    let (base, np_enc) = NoProtoBench::encode_bench().unwrap();
    let fb_enc = FlatBufferBench::encode_bench(base);
    let bn_enc = BincodeBench::encode_bench(base);
    let pb_enc = ProtocolBufferBench::encode_bench(base);
    let msg_enc = MessagePackBench::encode_bench(base);
    let json_enc = JSONBench::encode_bench(base);
    let bson_enc = BSONBench::encode_bench(base);
    let pro_enc = ProstBench::encode_bench(base);
    let avro_enc = AvroBench::encode_bench(base);
    let flx_enc = FlexBench::encode_bench(base);
    let abo_enc = AbomBench::encode_bench(base);
    // let rbso_enc = RawBSONBench::encode_bench(base);

    println!("\n======== DECODE BENCHMARK ========");

    let (base, np_dec) = NoProtoBench::decode_bench().unwrap();
    let fb_dec = FlatBufferBench::decode_bench(base);
    let bn_dec = BincodeBench::decode_bench(base);
    let pb_dec = ProtocolBufferBench::decode_bench(base);
    let msg_dec = MessagePackBench::decode_bench(base);
    let json_dec = JSONBench::decode_bench(base);
    let bson_dec = BSONBench::decode_bench(base);
    let pro_dec = ProstBench::decode_bench(base);
    let avro_dec = AvroBench::decode_bench(base);
    let flx_dec = FlexBench::decode_bench(base);
    let abo_dec = AbomBench::decode_bench(base);
    // let rbso_dec = RawBSONBench::decode_bench(base);

    println!("\n====== DECODE ONE BENCHMARK ======");

    let (base, np_dec1) = NoProtoBench::decode_one_bench().unwrap();
    let fb_dec1 = FlatBufferBench::decode_one_bench(base);
    let bn_dec1 = BincodeBench::decode_one_bench(base);
    let pb_dec1 = ProtocolBufferBench::decode_one_bench(base);
    let msg_dec1 = MessagePackBench::decode_one_bench(base);
    let json_dec1 = JSONBench::decode_one_bench(base);
    let bson_dec1 = BSONBench::decode_one_bench(base);
    let pro_dec1 = ProstBench::decode_one_bench(base);
    let avro_dec1 = AvroBench::decode_one_bench(base);
    let flx_dec1 = FlexBench::decode_one_bench(base);
    let abo_dec1 = AbomBench::decode_one_bench(base);
    // let rbso_dec1 = RawBSONBench::decode_one_bench(base);

    println!("\n====== UPDATE ONE BENCHMARK ======");

    let (base, np_up) = NoProtoBench::update_bench().unwrap();
    let fb_up = FlatBufferBench::update_bench(base);
    let bn_up = BincodeBench::update_bench(base);
    let pb_up = ProtocolBufferBench::update_bench(base);
    let msg_up = MessagePackBench::update_bench(base);
    let json_up = JSONBench::update_bench(base);
    let bson_up = BSONBench::update_bench(base);
    let pro_up = ProstBench::update_bench(base);
    let avro_up = AvroBench::update_bench(base);
    let flx_up = FlexBench::update_bench(base);
    let abo_up = AbomBench::update_bench(base);
    // let rbso_up = RawBSONBench::update_bench(base);

    println!("\n\n");


    println!("//! | Library            | Encode | Decode All | Decode 1 | Update 1 | Size (bytes) | Size (Zlib) |");
    println!("//! |--------------------|--------|------------|----------|----------|--------------|-------------|");
    println!("//! | **Runtime Libs**   |        |            |          |          |              |             |");
    println!("//! | *NoProto*          | {} |     {} |   {} |   {} |          {} |         {} |", np_enc, np_dec, np_dec1, np_up, np_size.0, np_size.1);
    println!("//! | Apache Avro        | {} |     {} |   {} |   {} |          {} |         {} |", avro_enc, avro_dec, avro_dec1, avro_up, avro_size.0, avro_size.1);
    println!("//! | FlexBuffers        | {} |     {} |   {} |   {} |          {} |         {} |", flx_enc, flx_dec, flx_dec1, flx_up, flx_size.0, flx_size.1);
    println!("//! | JSON               | {} |     {} |   {} |   {} |          {} |         {} |", json_enc, json_dec, json_dec1, json_up, json_size.0, json_size.1);
    println!("//! | BSON               | {} |     {} |   {} |   {} |          {} |         {} |", bson_enc, bson_dec, bson_dec1, bson_up, bson_size.0, bson_size.1);
    // println!("//! | Raw BSON           | {} |     {} |   {} |   {} |          {} |         {} |", rbso_enc, rbso_dec, rbso_dec1, rbso_up, rbso_size.0, rbso_size.1);
    println!("//! | MessagePack        | {} |     {} |   {} |   {} |          {} |         {} |", msg_enc, msg_dec, msg_dec1, msg_up, msg_size.0, msg_size.1);
    println!("//! | **Compiled Libs**  |        |            |          |          |              |             |");
    println!("//! | Flatbuffers        | {} |     {} |   {} |   {} |          {} |         {} |", fb_enc, fb_dec, fb_dec1, fb_up, fb_size.0, fb_size.1);
    println!("//! | Bincode            | {} |     {} |   {} |   {} |          {} |         {} |", bn_enc, bn_dec, bn_dec1, bn_up, bn_size.0, bn_size.1);
    println!("//! | Protobuf           | {} |     {} |   {} |   {} |          {} |         {} |", pb_enc, pb_dec, pb_dec1, pb_up, pb_size.0, pb_size.1);
    println!("//! | Prost              | {} |     {} |   {} |   {} |          {} |         {} |", pro_enc, pro_dec, pro_dec1, pro_up, pro_size.0, pro_size.1);
    println!("//! | Abomonation        | {} |     {} |   {} |   {} |          {} |         {} |", abo_enc, abo_dec, abo_dec1, abo_up, abo_size.0, abo_size.1);
}

