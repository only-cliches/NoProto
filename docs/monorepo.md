# NoProto Monorepo

NoProto is moving from a single Rust crate repository toward a language implementation monorepo. The Rust crate remains the reference implementation at version `0.9.60` while the shared specification and conformance suite are extracted.

## Layout

- `no_proto_rs/` contains the Rust crate published as `no_proto`.
- `no_proto_js/` contains the existing Rust/WASM JavaScript package.
- `packages/javascript/` is reserved for a native JavaScript or TypeScript implementation.
- `packages/php/` is reserved for a native PHP implementation.
- `packages/go/` is reserved for a native Go implementation.
- `spec/` contains the language-neutral format and schema specification.
- `conformance/` contains shared test vectors and compatibility requirements.
- `bench/` contains the existing Rust benchmark suite.

## Implementation Rules

Every language package should implement the behavior described by `spec/` and validate against `conformance/` before release.

The Rust crate is the compatibility anchor until the specification is complete. If a behavior is not yet documented in `spec/`, match `no_proto_rs` version `0.9.60`.

Language packages should not introduce incompatible buffer, schema, or JSON conversion behavior without first updating the specification and adding conformance coverage.

## Versioning

Package versions should track the NoProto compatibility level they implement. A package that targets the stable Rust crate behavior should start from the `0.9.60` compatibility line, even if its own package ecosystem requires a different release format.
