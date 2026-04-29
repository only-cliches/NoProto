# NoProto Specification

This directory is the home for the language-neutral NoProto specification.

The current compatibility baseline is `no_proto_rs` version `0.9.60`. Until the full specification is extracted, the Rust implementation is authoritative for behavior that is not yet documented here.

## Scope

The specification should cover:

- Buffer layout and pointer encoding.
- Supported scalar and collection types.
- Schema syntax and schema binary representation.
- Default values and optional field behavior.
- JSON import and export semantics.
- Sortable byte encoding.
- Error handling expectations for invalid buffers.

## Compatibility

All language implementations must read and write buffers that are compatible with the `0.9.60` Rust crate unless a future spec version explicitly changes the format.

Spec changes that affect encoded bytes need matching fixtures in `../conformance/`.
