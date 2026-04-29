# NoProto Conformance

This directory is for shared compatibility fixtures used by every language implementation.

The conformance suite should prove that each implementation can parse the same schemas, encode the same values, decode the same buffers, and export the same JSON-compatible data as `no_proto_rs` `0.9.60`.

## Fixture Shape

Use one directory per case:

```text
conformance/cases/<case-name>/
  schema.json
  schema.idl
  input.json
  buffer.hex
  output.json
```

- `schema.json` is the canonical schema when JSON schema syntax is available.
- `schema.idl` is the IDL form when the case covers IDL parsing.
- `input.json` is the value to encode.
- `buffer.hex` is the canonical encoded buffer as lowercase hexadecimal.
- `output.json` is the expected exported value after decoding.

Language packages should load these fixtures directly instead of duplicating them.
