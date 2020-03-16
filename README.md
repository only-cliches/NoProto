Note: initial library was conceived in javascript, haven't finished porting the documentation over yet.

```rs
// utf-8 string from vec<u8>
let buf = &[0x41u8, 0x41u8, 0x42u8];
let s = String::from_utf8_lossy(buf);

// move utf-8 string into vec<u8>
https://doc.rust-lang.org/std/string/struct.String.html#method.into_bytes

// setting numbers in vector
https://docs.rs/byteorder/1.3.2/byteorder/

// data models
https://docs.rs/json/0.12.0/json/
```


```ts
const model = {
    kind: "table",
    columns: [
        ["name", {kind: "string"}],
        ["scores", {kind: "tuple", size: 3, values: {kind: "u64"}}]
        ["favs", {kind: "list", of: {
            kind: "table",
            columns: [
                ...
            ]
        }}],
        ["meta", {kind: "map", key: {kind: "string"}, value: {kind: "string"}}],
        ["colors", {kind: "option", options: ["red", "blue", "green"]}]
    ]
}
```

# NP_
High Performance Serialization Library

## Features
- Nearly instant deserilization & serialization
- Schemas are dynamic/flexible at runtime
- Mutate/Update/Delete values in existing buffers
- Supports native data types
- Supports collection types (list, map, & table)

NP_ allows you to store and mutate structured data with near zero overhead.  It's like JSON but faster, type safe and more space efficient.

NP_ moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.  Deserilizing is free, exporting just gives you the buffer created by the library.

#### Compared to FlatBuffers & Cap'N Proto:
- Schemas are dynamic at runtime, no compilation step
- Supports more types and better nested type support
- Mutate (add/delete/update) existing/imported buffers

#### Compared to JSON
- More space efficient
- Has schemas
- Faster serialization & deserialization
- Supports raw bytes & other native types

#### Compared to BSON
- Faster serialization & deserialization
- Has schemas
- Typically (but not always) more space efficient
- Supports much larger documents (4GB vs 16MB)
- Better collection support & more supported types

### Good Use Cases For NP_
- Database object storage.
- File object storage.
- Sending/Receiving objects over a network.

### Compaction
To keep performance optimal many mutations/updates/deletes will lead to the buffer occupying more space than the data it contains.  This space can be recovered with compaction.

For example, if you set a key `baz` to `"bar"`, the value will just be appended to the buffer (no compaction needed).  If you then update the key `baz` to `"hello"`, the `"hello"` string will be appended to the buffer and `baz` will point to the new value.  The old value, `"bar"` is still in the buffer and taking up space but has been dereferenced.

Fixed size scalar values are an exception to this rule, they can always be updated in place and never take up more space following an update.

Deletes are always done by simply dereferencing data, so deleting a value will always lead to wasted space.

The library can inexpensively calculate how much space is wasted with the `.getWastedBytes()` method, and compactions can be performed with `.compact()` and `.maybeCompact(callback: (wastedBytes) => boolean)`.  Compaction creates a new buffer and copies over all data from the old buffer skipping any dereferenced data.

In a server/client setup compaction can be performed in batches on the server during non peak times.  Compaction can also be handled by the client before sending the updated buffer to the server.




## Example
```ts
import * as NP_ from "NP_";

const dataModels = {
    user: [
        ["id",        "uuid", {i: 0}],
        ["first",   "string", {i: 1}],
        ["last",    "string", {i: 2}],
        ["email",    "email", {i: 3}],
        ["age",        "int", {i: 4, default: 13}],
        ["address",  "table", {i: 5, model: [
            ["street",   "string", {i: 0}],
            ["street2",  "string", {i: 1}],
            ["city",     "string", {i: 2}],
            ["state",    "string", {i: 3}],
            ["zip",      "string", {i: 4}]
        ]}],
        ["tags",  "string[]", {i: 6}]
    ]
    post: [
        ["id",        "uuid", {i: 0}],
        ["author",    "uuid", {i: 1}],
        ["title",   "string", {i: 2}],
        ["body",    "string", {i: 3}]
    ]
}

// new object
const newUser = NP_.makeRoot(dataModels.user);

// Basically ES6 Map API for tables
newUser.root.set("id", "5f1e1667-ff4d-4910-94e0-30a7d7e00310");
newUser.root.set("first", "Sherlock",);
newUser.root.set("last",  "Holmes");

// make new nested table
const address: NP_.Table = newUser.root.set("address", NP_.makeTable()); 
address.set("street", "221B Baker St");

// make new list
const tags: NP_.List<string> = newUser.root.set("tags", NP_.makeList());
// lists have Array like API
tags.push("my", "tags", "here");
tags.unshift("more");


// deserialize
const buffer: Uint8Array[] = newUser.export();
// write to disk: fs.writeFileSync("test.npro", Buffer.concat(buffer));
// export to json: newUser.toJSON();

// serialize
const importedUser = NP_.makeRoot(dataModels.user, buffer);
console.log(importedUser.root.get("first")) // "SherlocK"

const importedAddress = importedUser.get("address");
console.log(importedAddress.get("street")) // "221B Baker St"


```


```js

// example data model
[
    ["id",       "uuid",     {i: 0, pk: true}], // i is optional, explicit index declaration
    ["username", "string",   {i: 1, immutable: true, login: true}],
    ["email",    "string",   {login: true}],
    ["pass",     "string",   {default: "", hidden: true, pass: "scrypt"}],
    ["tags",     "string[]", {}],
    ["age",      "int",      {max: 130, min: 13, default: 0, notNull: true}],
    ["type",     "enum",     {options: [
        "admin", "user", "none"
    ]}],
    ["address",   "table",     {model: [
        ["street",   "string"],
        ["street2",  "string"],
        ["city",     "string"],
        ["state",    "string"],
        ["zip",      "string"]
    ]}],
    ["meta",     "map<string, string>"]
]

// example buffer (Uint8Array)
[

    // map data type
    0, 0, 0, 0 // offset of first key (Uint32)
    ... // anything

    0, 0, 0 ,0, // offset of next key/value (zero if no other keys)
    0,// index of value in data model (Uint8)
    ... // value data (specific to type)

    // list data type
    0, 0, 0, 0 // offset of first value (Uint32)
    0, 0, 0, 0 // offset of last value (Uint32)
    ... // anything

    0, 0, 0, 0, // offset of previous value (zero if beginning)
    0, 0, 0, 0, // offset of next value (zero if end)
    ... // data of type

    // string data type
    0, 0, 0, 0 // size of data (Uint32)
    ... // string data (utf-8)
    // convert string to Uint8Array 
    // const array = new TextEncoder("utf-8").encode("string");
    // convert Uint8array to string
    // const str = new TextDecoder("utf-8").decode(array);
    // https://www.npmjs.com/package/text-encoding-shim

    // boolean type
    0, // 0 or 1 (Uint8)

    // int type (64 bit signed integer) (8 bytes)
    4, // value type (Uint8)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0

    // float type (long double, 64 bit assumed)
    5, // value type (Uint8)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0

]

/*
supported data types:
int (int64), uint8, uint16, uint32, uint64, int8, int16, int32, int64
float (float64), float32, float64
string (utf-8 string)
bytes (collection of bytes)
enum
table (map/object)
map(string, string)
boolean/bool
geo (lat, lon)
// 0 (3.5nm resolution): two 64 bit float (16 bytes)
// 1 (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
// 2 (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
uuid (32 byte random number)
date (ISO8601 date field) (32 bit uint)
time_id (4 byte uint timestamp) + (16 byte random number)

*/

// 4 bytes to 32 bit integer:
// c[0] + (c[1] << 8) + (c[2] << 16) + (c[3] << 24)

// API
import * as NP_ from "NP_";

// parse object (Basically JS Map API)
const user = NP_.load([ /* ArrayBuffer or Uint8Array */ ], dataModel);
user.get("name"); // get value
user.set("name", "value"); // set value

// list api

const newList = user.set("tags", NP_.makeList()); // replace/create list
// const existingList = user.get("tags");
newList.push(...) // add to end of array
newList.unshift(...) // add to beginning of array
newList.pop() // remove last element and return it
newList.shift() // remove first element and return it

newList.map(...)
newList.filter(...)
newList.forEach(...)

const it = newList.iterator();
// const it = newList.reverseIterator();
while(it.hasValue()) {
    const val = it.value();
    it.set(/* set value here */);
    it.remove(); // remove value here
    it.index(); // current index

    it.next();
}

const newMap = user.set("address", NP_.makeMap()); // replace/create map
// const existingMap = user.get("address");

user.getWastedBytes() // how many bytes can we free if we compact?

// compact object  (optional)
user.compact();

// get bytes of object
const bytes: Uint8Array[] = user.export(); 

// create new object
const newUser = NP_.makeRoot(dataModel);
newUser.set("name", "value");
newUser.export();

// recursively convert to JSON object
user.toJSON(); 
// serialize JSON into NP_
const newUser2 = NP_.fromJSON(jsonObject, dataModel);

```

MIT License

Copyright (c) 2019 Scott Lott

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.