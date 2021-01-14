// WIP IDL ideas...

// this is a comment
struct({
    name: string({default: "default value here", size: 20}),
    tags: list({of: string()}),
    tuple: tuple({sorted: true, values: [string(), string(), string()]}),
    map: map({values: string()}),
    // another comment
    enum: option({default: "red", choices: ["red", "blue", "orange"]}),
    p: portal({to: "map"}),
    nested: struct({
        name: string(),
        value: u32({default: 20}),
        geo: geo({size: 4, default: {lat: 20, lng: 20.28}}),
    })
});



rpc_spec({
    name: "Test API",
    author: "hello",
    version: "1.0.0",
    spec: (mod, self) => {
        msg("send_name", struct());
        rpc("your_face", fn(self.send_name), option(self.send_name));
        rpc("your_face", fn(self.argument), result(self.send_name, self.error));
        rpc("your_face", fn(self.argument), empty());
        mod("mod_name", (self) => {

        });
    }
});

struct({
    name: string(),
    age:  u16({default: 0}),
    tags: list({of: string()})
});

//! let user_factory = NP_Factory::new(r#"{
//!     "type": "struct",
//!     "fields": [
//!         ["name",   {"type": "string"}],
//!         ["age",    {"type": "u16", "default": 0}],
//!         ["tags",   {"type": "list", "of": {
//!             "type": "string"
//!         }}]
//!     ]
//! }"#)?;