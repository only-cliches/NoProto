# Language Packages

This directory is reserved for native NoProto implementations in additional languages.

Current package slots:

- `javascript/` for a native JavaScript or TypeScript implementation.
- `php/` for a native PHP implementation.
- `go/` for a native Go implementation.

Each package should include its own package manager metadata, tests, examples, and release notes. Shared wire-format behavior belongs in `../spec/`, and shared compatibility cases belong in `../conformance/`.

The existing `../no_proto_js/` package is a Rust/WASM package and remains separate from the future native JavaScript implementation.
