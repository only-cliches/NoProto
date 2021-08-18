use crate::memory::NP_Memory;
use crate::error::NP_Error;
use alloc::prelude::v1::{Vec, String, ToString};
use alloc::sync::Arc;
use crate::schema::{NP_Schema, NP_Schema_Type};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum NP_Types {
    none,
    any,
    string,
    char,
    i8, i16, i32, i64,
    u8, u16, u32, u64,
    f32, f64,
    dec32, dec64,
    bool,
    geo32, geo64, geo128,
    uuid, ulid,
    date,
    vec, map, _box, result, option,
    tuple { len: u8 },
    array { len: u16 },
    small_array { len: u8 },
    custom { idx: usize },
    small_custom { idx: usize },
    rpc_request { idx: usize, func: usize },
    rpc_response { idx: usize, func: usize },
}

impl From<&str> for NP_Types {
    fn from(value: &str) -> NP_Types {
        match value.trim() {
            "any" => NP_Types::any,
            "string" => NP_Types::string,
            "char" => NP_Types::char,
            "i8" => NP_Types::i8,
            "i16" => NP_Types::i16,
            "i32" => NP_Types::i32,
            "i64" => NP_Types::i64,
            "u8" => NP_Types::u8,
            "u16" => NP_Types::u16,
            "u32" => NP_Types::u32,
            "u64" => NP_Types::u64,
            "f32" => NP_Types::f32,
            "f64" => NP_Types::f64,
            "dec32" => NP_Types::dec32,
            "dec64" => NP_Types::dec64,
            "bool" => NP_Types::bool,
            "geo32" => NP_Types::geo32,
            "geo64" => NP_Types::geo64,
            "geo128" => NP_Types::geo128,
            "uuid" => NP_Types::uuid,
            "ulid" => NP_Types::ulid,
            "date" => NP_Types::date,
            "Vec" => NP_Types::vec,
            "Map" => NP_Types::map,
            "Box" => NP_Types::_box,
            "Result" => NP_Types::result,
            "Option" => NP_Types::option,
            _ => NP_Types::custom { idx: 0 }
        }
    }
}

impl Into<&str> for NP_Types {
    fn into(self) -> &'static str {
        match self {
            NP_Types::none     =>  "none",
            NP_Types::any      =>  "any",
            NP_Types::string   =>  "string",
            NP_Types::char     =>  "char",
            NP_Types::i8       =>  "i8",
            NP_Types::i16      =>  "i16",
            NP_Types::i32      =>  "i32",
            NP_Types::i64      =>  "i64",
            NP_Types::u8       =>  "u8",
            NP_Types::u16      =>  "u16",
            NP_Types::u32      =>  "u32",
            NP_Types::u64      =>  "u64",
            NP_Types::f32      =>  "f32",
            NP_Types::f64      =>  "f64",
            NP_Types::dec32    =>  "dec32",
            NP_Types::dec64    =>  "dec64",
            NP_Types::bool     =>  "bool",
            NP_Types::geo32    =>  "geo32",
            NP_Types::geo64    =>  "geo64",
            NP_Types::geo128   =>  "geo128",
            NP_Types::uuid     =>  "uuid",
            NP_Types::ulid     =>  "ulid",
            NP_Types::date     =>  "date",
            NP_Types::vec      =>  "Vec",
            NP_Types::map      =>  "Map",
            NP_Types::_box     =>  "Box",
            NP_Types::result   =>  "Result",
            NP_Types::option   =>  "Option",
            NP_Types::array  { .. } => "array",
            NP_Types::custom { .. } => "custom",
            NP_Types::small_custom { .. } => "custom",
            NP_Types::small_array { .. } => "array",
            NP_Types::tuple { .. } => "tuple",
            NP_Types::rpc_request { .. } => "rpc",
            NP_Types::rpc_response { .. } => "rpc"
        }
    }
}

impl From<NP_Types> for u8 {
    fn from(np_type: NP_Types) -> Self {
        match np_type {
            NP_Types::none     =>  0,
            NP_Types::any      =>  1,
            NP_Types::string   =>  2,
            NP_Types::char     =>  3,
            NP_Types::i8       =>  4,
            NP_Types::i16      =>  5,
            NP_Types::i32      =>  6,
            NP_Types::i64      =>  7,
            NP_Types::u8       =>  8,
            NP_Types::u16      =>  9,
            NP_Types::u32      => 10,
            NP_Types::u64      => 11,
            NP_Types::f32      => 12,
            NP_Types::f64      => 13,
            NP_Types::dec32    => 14,
            NP_Types::dec64    => 15,
            NP_Types::bool     => 16,
            NP_Types::geo32    => 17,
            NP_Types::geo64    => 18,
            NP_Types::geo128   => 19,
            NP_Types::uuid     => 20,
            NP_Types::ulid     => 21,
            NP_Types::date     => 22,
            NP_Types::vec      => 23,
            NP_Types::map      => 24,
            NP_Types::_box     => 25,
            NP_Types::result   => 26,
            NP_Types::option   => 27,
            NP_Types::array  { .. } => 28,
            NP_Types::custom { .. } => 29,
            NP_Types::small_custom { .. } => 30,
            NP_Types::small_array { .. } => 31,
            NP_Types::tuple { .. }=> 32,
            NP_Types::rpc_request { .. } => 33,
            NP_Types::rpc_response { .. } => 34
        }
    }
}

impl From<u8> for NP_Types {
    fn from(byte: u8) -> NP_Types {
        match byte {
            0  => NP_Types::none,
            1  => NP_Types::any,
            2  => NP_Types::string,
            3  => NP_Types::char,
            4  => NP_Types::i8,
            5  => NP_Types::i16,
            6  => NP_Types::i32,
            7  => NP_Types::i64,
            8  => NP_Types::u8,
            9  => NP_Types::u16,
            10 => NP_Types::u32,
            11 => NP_Types::u64,
            12 => NP_Types::f32,
            13 => NP_Types::f64,
            14 => NP_Types::dec32,
            15 => NP_Types::dec64,
            16 => NP_Types::bool,
            17 => NP_Types::geo32,
            18 => NP_Types::geo64,
            19 => NP_Types::geo128,
            20 => NP_Types::uuid,
            21 => NP_Types::ulid,
            22 => NP_Types::date,
            23 => NP_Types::vec,
            24 => NP_Types::map,
            25 => NP_Types::_box,
            26 => NP_Types::result,
            27 => NP_Types::option,
            28 => NP_Types::array { len: 0 },
            29 => NP_Types::custom { idx: 0 },
            30 => NP_Types::small_custom { idx: 0 },
            31 => NP_Types::small_array { len : 0},
            32 => NP_Types::tuple { len: 0 },
            33 => NP_Types::rpc_request { idx: 0, func: 0 },
            34 => NP_Types::rpc_response { idx: 0, func: 0 },
            _ => NP_Types::none
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum buffer_rpc {
    none,
    request,
    response
}

#[derive(Debug, Clone, PartialEq)]
pub struct NP_Buffer_Type {
    pub kind: NP_Types,
    pub generics: Option<Vec<NP_Buffer_Type>>
}

impl NP_Buffer_Type {

    pub fn generate_string(&self, schema: &Arc<NP_Schema>) -> String {
        let mut result = String::from("");

        let type_str: &str = self.kind.into();

        let mut is_array: bool = false;

        match type_str {
            "tuple" => {
                is_array = true;
                result.push_str("(");
                if let Some(generics) = &self.generics {
                    result.push_str(generics.iter().map(|item| item.generate_string(&schema)).collect::<Vec<String>>().join(", ").as_str());
                }
                result.push_str(")");
            }
            "rpc" => {
                if let NP_Types::rpc_request { idx, func } = self.kind {
                    if let Some(type_data) = schema.id_index.get(idx) {
                        let parsed_schema = &schema.schemas[type_data.data];
                        if let Some(name) = parsed_schema.name {
                            result.push_str(name.read_bytes(&schema.source));
                        }
                    }
                }
                if let NP_Types::rpc_response { idx, func } = self.kind {
                    if let Some(type_data) = schema.id_index.get(idx) {
                        let parsed_schema = &schema.schemas[type_data.data];
                        if let Some(name) = parsed_schema.name {
                            result.push_str(name.read_bytes(&schema.source));
                        }
                    }
                }
            },
            "array" => {
                is_array = true;
                result.push_str("[");
                if let Some(gen) = &self.generics {
                    result.push_str(gen[0].generate_string(&schema).as_str());
                    result.push_str("; ");
                }
                if let NP_Types::array { len } = self.kind {
                    result.push_str(len.to_string().as_str());
                }
                if let NP_Types::small_array { len } = self.kind {
                    result.push_str(len.to_string().as_str());
                }
                result.push_str("]");
            },
            "custom" => {
                if let NP_Types::custom { idx } = self.kind {
                    if let Some(type_data) = schema.id_index.get(idx) {
                        let parsed_schema = &schema.schemas[type_data.data];
                        if let Some(name) = parsed_schema.name {
                            result.push_str(name.read_bytes(&schema.source));
                        }
                    }
                }
                if let NP_Types::small_custom { idx } = self.kind {
                    if let Some(type_data) = schema.id_index.get(idx) {
                        let parsed_schema = &schema.schemas[type_data.data];
                        if let Some(name) = parsed_schema.name {
                            result.push_str(name.read_bytes(&schema.source));
                        }
                    }
                }
            },
            _=> {
                result.push_str(type_str);
            }
        }

        if is_array == false {
            if let Some(generics) = &self.generics {
                result.push_str("<");
                result.push_str(generics.iter().map(|item| item.generate_string(&schema)).collect::<Vec<String>>().join(", ").as_str());
                result.push_str(">");
            }
        }

        if type_str == "rpc" {
            result.push_str(".");
            if let NP_Types::rpc_request { idx, func } = self.kind {
                if let Some(type_data) = schema.id_index.get(idx) {
                    if let Some(methods) = type_data.methods {
                        if let NP_Schema_Type::Impl { children } = &schema.schemas[methods].data_type {
                            for (id, (hash, key)) in children.keys().iter().enumerate() {
                                if id == func {
                                    result.push_str(key.as_str());
                                }
                            }
                        }
                    }

                }
            }
            if let NP_Types::rpc_response { idx, func } = self.kind {
                if let Some(type_data) = schema.id_index.get(idx) {
                    if let Some(methods) = type_data.methods {
                        if let NP_Schema_Type::Impl { children } = &schema.schemas[methods].data_type {
                            for (id, (hash, key)) in children.keys().iter().enumerate() {
                                if id == func {
                                    result.push_str(key.as_str());
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    pub fn get_bytes(&self) -> Result<(u8, [u8; 24]), NP_Error> { // length, (bytes)
        let mut length = 1usize;
        let mut bytes: [u8; 24] = Default::default();

        bytes[0] = self.kind.into();

        if let NP_Types::array { len } = &self.kind {
            let u16_bytes = len.to_be_bytes();
            bytes[length] = u16_bytes[0];
            bytes[length + 1] = u16_bytes[1];
            length += 2;
        }

        if let NP_Types::small_array { len } = &self.kind {
            bytes[length] = *len;
            length += 1;
        }

        if let NP_Types::custom { idx } = &self.kind {
            let u16_bytes = (*idx as u16).to_be_bytes();
            bytes[length] = u16_bytes[0];
            bytes[length + 1] = u16_bytes[1];
            length += 2;
        }

        if let NP_Types::rpc_request { idx, func } = &self.kind {
            let u16_bytes = (*idx as u16).to_be_bytes();
            bytes[length] = u16_bytes[0];
            bytes[length + 1] = u16_bytes[1];
            length += 2;

            bytes[length] = *func as u8;
            length += 1;
        }

        if let NP_Types::rpc_response { idx, func } = &self.kind {
            let u16_bytes = (*idx as u16).to_be_bytes();
            bytes[length] = u16_bytes[0];
            bytes[length + 1] = u16_bytes[1];
            length += 2;

            bytes[length] = *func as u8;
            length += 1;
        }

        if let NP_Types::small_custom { idx } = &self.kind {
            bytes[length] = *idx as u8;
            length += 1;
        }

        if let NP_Types::tuple { len } = &self.kind {
            bytes[length] = *len;
            length += 1;
        }

        if let Some(generics) = &self.generics {
            for (_idx, g) in generics.iter().enumerate() {
                let (new_length , new_bytes) = g.get_bytes()?;
                if new_length as usize + length >= bytes.len() {
                    return Err(NP_Error::Custom { message: String::from("Too many buffer types, buffer schema overflow!") })
                }
                let mut i: usize = 0;
                while i < new_length as usize {
                    bytes[length] = new_bytes[i];
                    length += 1;
                    i += 1;
                }
            }
        }

        Ok((length as u8, bytes))
    }

    pub fn from_bytes(bytes: &[u8], schema: &Arc<NP_Schema>) -> Result<(usize, Self), NP_Error> {

        if bytes.len() == 0 {
            return Err(NP_Error::OutOfBounds)
        }

        let mut index = 0usize;

        let mut kind: NP_Types = bytes[index].into();

        index += 1;

        match &mut kind {
            NP_Types::tuple { len } => {
                if bytes.len() < index + 1 {
                    return Err(NP_Error::OutOfBounds)
                }

                *len = bytes[index];
                index += 1;
            },
            NP_Types::small_custom { idx } => {
                if bytes.len() < index + 1 {
                    return Err(NP_Error::OutOfBounds)
                }

                *idx = bytes[index] as usize;
                index += 1;
            },
            NP_Types::rpc_request { idx, func } => {
                if bytes.len() < index + 3 {
                    return Err(NP_Error::OutOfBounds)
                }

                let mut u16_bytes: [u8; 2] = Default::default();
                u16_bytes[0] = bytes[index];
                u16_bytes[1] = bytes[index + 1];
                index += 2;
                *idx = u16::from_be_bytes(u16_bytes) as usize;

                *func = bytes[index] as usize;
                index += 1;
            },
            NP_Types::rpc_response { idx, func } => {
                if bytes.len() < index + 3 {
                    return Err(NP_Error::OutOfBounds)
                }

                let mut u16_bytes: [u8; 2] = Default::default();
                u16_bytes[0] = bytes[index];
                u16_bytes[1] = bytes[index + 1];
                index += 2;
                *idx = u16::from_be_bytes(u16_bytes) as usize;

                *func = bytes[index] as usize;
                index += 1;
            },
            NP_Types::custom { idx } => {
                if bytes.len() < index + 2 {
                    return Err(NP_Error::OutOfBounds)
                }

                let mut u16_bytes: [u8; 2] = Default::default();
                u16_bytes[0] = bytes[index];
                u16_bytes[1] = bytes[index + 1];
                index += 2;
                *idx = u16::from_be_bytes(u16_bytes) as usize;
            },
            NP_Types::array { len } => {
                if bytes.len() < index + 2 {
                    return Err(NP_Error::OutOfBounds)
                }

                let mut u16_bytes: [u8; 2] = Default::default();
                u16_bytes[0] = bytes[index];
                u16_bytes[1] = bytes[index + 1];
                index += 2;
                *len = u16::from_be_bytes(u16_bytes);
            },
            NP_Types::small_array { len } => {
                if bytes.len() < index + 1 {
                    return Err(NP_Error::OutOfBounds)
                }

                *len = bytes[index];
                index += 1;
            }
            _ => { }
        }

        if kind == NP_Types::none {
            return Err(NP_Error::Custom { message: String::from("Error parsing buffer type: unknown data type!") })
        }

        let mut generics: Option<Vec<NP_Buffer_Type>> = None;

        let mut generic_length: usize = Self::read_generic_length(&kind, schema);

        // parse generics
        if generic_length > 0 {
            generics = Some(Vec::new());
            while generic_length > 0 {
                if let Some(gener) = &mut generics {
                    let (add_len, parsed) = Self::from_bytes(&bytes[index..bytes.len()], schema)?;
                    index += add_len;
                    gener.push(parsed);
                }
                generic_length -= 1;
            }
        }

        Ok((index, Self {
            kind,
            generics
        }))
    }

    #[inline(always)]
    fn read_generic_length(kind: &NP_Types, schema: &Arc<NP_Schema>) -> usize {
        match kind {
            NP_Types::vec => 1,
            NP_Types::map => 1,
            NP_Types::_box => 1,
            NP_Types::result => 2,
            NP_Types::option => 1,
            NP_Types::array { .. } => 1,
            NP_Types::small_array { .. } => 1,
            NP_Types::tuple { len } => *len as usize,
            NP_Types::small_custom { idx } => {
                if let Some(custom) = schema.id_index.get(*idx) {
                    let custom_type = &schema.schemas[custom.data];
                    if let Some(generics) = &custom_type.self_generics {
                        generics.1.len()
                    } else {
                        0
                    }
                } else {
                    0
                }
            },
            NP_Types::custom { idx } => {
                if let Some(custom) = schema.id_index.get(*idx) {
                    let custom_type = &schema.schemas[custom.data];
                    if let Some(generics) = &custom_type.self_generics {
                        generics.1.len()
                    } else {
                        0
                    }
                } else {
                    0
                }
            },
            _ => 0,
        }
    }

    pub fn parse_type_prc(rpc_type: &buffer_rpc, data_type: &str, schema: &Arc<NP_Schema>) -> Result<Option<Self>, NP_Error> {

        let mut dot_pos: Option<usize> = None;

        for (idx, char) in data_type.chars().enumerate() {
            if char == '.' {
                if None == dot_pos {
                    dot_pos = Some(idx);
                } else {
                    return Err(NP_Error::Custom { message: String::from("Multiple dot paths detected in rpc call.") })
                }
            }
        }

        if let Some(idx) = dot_pos {
            let mut root_type = NP_Error::unwrap(Self::parse_type(rpc_type, &data_type[0..idx], schema)?)?;
            let method_name = &data_type[(idx + 1)..data_type.len()];

            Ok(Some(root_type))
        } else {
            Err(NP_Error::Custom { message: String::from("No method call found in rpc request!") })
        }
    }

    pub fn parse_type(is_rpc: &buffer_rpc, data_type: &str, schema: &Arc<NP_Schema>) -> Result<Option<Self>, NP_Error> {

        if data_type.len() > 255 {
            return Err(NP_Error::Custom { message: String::from("Buffer schemas cannot be longer than 255 characters!") })
        }

        if data_type.trim() == "" {
            return Ok(None);
        }

        // unit type
        if data_type.trim() == "()" {
            return Ok(Some(Self {
                kind: NP_Types::tuple { len : 0 },
                generics: None
            }));
        }

        let mut has_generics = false;
        let mut angle_counter: isize = 0;
        for char in data_type.chars() {
            if char == '<' || char == '[' || char == '(' {
                angle_counter += 1;
                has_generics = true;
            }
            if char == '>' || char == ']' || char == ')' {
                angle_counter -= 1;
            }
        }

        if angle_counter != 0 {
            return Err(NP_Error::Custom { message: String::from("Missing matching brackets!")})
        }

        if has_generics { // slow path :(

            let mut result = Self {
                kind: NP_Types::none,
                generics: Some(Vec::new())
            };

            #[derive(Debug, PartialEq)]
            enum parse_state {
                searching,
                angle_bracket,
                square_bracket,
                parans
            }

            let mut angle_step = 0isize;
            let mut square_step = 0isize;
            let mut paran_step = 0isize;

            let mut p_state = parse_state::searching;

            let mut parse_cursor: (usize, usize) = (0, 0); // (start_idx, end_idx)

            let chars = data_type.as_bytes();
            while parse_cursor.1 < data_type.len() {

                match chars[parse_cursor.1] as char {
                    '(' => {
                        if paran_step == 0 && p_state == parse_state::searching {
                            result.kind = NP_Types::tuple { len: 0 };
                            parse_cursor.0 = parse_cursor.1 + 1;
                            p_state = parse_state::parans;
                        }
                        paran_step += 1;
                    },
                    ')' => {
                        paran_step -= 1;

                        if paran_step == 0 && p_state == parse_state::parans {
                            let inner_type = Self::parse_type(is_rpc, data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
                            if let Some(generics) = &mut result.generics {
                                if let Some(i_type) = inner_type {
                                    generics.push(i_type);
                                }
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;

                            p_state = parse_state::searching;
                        }
                    },
                    '<' => { // generic xxx<xxx, xxxx, xxxx>

                        if angle_step == 0 && p_state == parse_state::searching {
                            let str_kind = data_type[parse_cursor.0..parse_cursor.1].trim();
                            result.kind = str_kind.into();

                            if let NP_Types::custom { idx } = &mut result.kind {
                                if let Some(custom_kind) = schema.name_index.get(str_kind) {
                                    if let Some(id) = schema.schemas[custom_kind.data].id {
                                        *idx = id as usize;
                                    }

                                } else {
                                    let mut msg = String::from("Unknown type found!: ");
                                    msg.push_str(str_kind);
                                    return Err(NP_Error::Custom { message: msg });
                                }
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;

                            p_state = parse_state::angle_bracket;
                        }

                        angle_step += 1;

                    },
                    '>' => {
                        angle_step -= 1;

                        if angle_step == 0 && p_state == parse_state::angle_bracket {
                            let inner_type = Self::parse_type(is_rpc, data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
                            if let Some(generics) = &mut result.generics {
                                if let Some(i_type) = inner_type {
                                    generics.push(i_type);
                                }
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;

                            p_state = parse_state::searching;
                        }
                    },
                    ';' => {
                        if square_step == 1 && p_state == parse_state::square_bracket {
                            parse_cursor.0 += 1;
                            let inner_type = Self::parse_type(is_rpc, data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
                            if let Some(generics) = &mut result.generics {
                                if let Some(i_type) = inner_type {
                                    generics.push(i_type);
                                }
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;
                        }
                    },
                    ',' => {
                        if paran_step == 0 && angle_step == 1 && p_state == parse_state::angle_bracket {
                            let inner_type = Self::parse_type(is_rpc, data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
                            if let Some(generics) = &mut result.generics {
                                if let Some(i_type) = inner_type {
                                    generics.push(i_type);
                                }
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;
                        }

                        if angle_step == 0 && paran_step == 1 && p_state == parse_state::parans {
                            let inner_type = Self::parse_type(is_rpc, data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
                            if let Some(generics) = &mut result.generics {
                                if let Some(i_type) = inner_type {
                                    generics.push(i_type);
                                }
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;
                        }
                    },
                    '[' => { // array [X; number]
                        if square_step == 0 && p_state == parse_state::searching {
                            parse_cursor.0 = parse_cursor.1;
                            p_state = parse_state::square_bracket;
                        }
                        square_step +=1;
                    },
                    ']' => {
                        square_step -=1;
                        if square_step == 0 && p_state == parse_state::square_bracket {
                            if let Ok(count) = data_type[parse_cursor.0..parse_cursor.1].trim().parse::<u16>() {
                                result.kind = NP_Types::array { len: count };
                            } else {
                                return Err(NP_Error::Custom { message: String::from("Error parsing array length!")})
                            }

                            parse_cursor.0 = parse_cursor.1 + 1;
                            p_state = parse_state::searching;
                        }
                    },
                    _ => { }
                }

                parse_cursor.1 += 1;
            }

            if square_step != 0 {
                return Err(NP_Error::Custom { message: String::from("Missing matching square brackets!")});
            }

            if angle_step != 0 {
                return Err(NP_Error::Custom { message: String::from("Missing matching angle brackets!")});
            }

            if paran_step != 0 {
                return Err(NP_Error::Custom { message: String::from("Missing matching parentheses!")});
            }

            let gen_count = if let Some(x) = &result.generics {
                x.len()
            } else {
                0
            };

            if gen_count == 0 {
                result.generics = None;
            }

            if let NP_Types::tuple { len } = &mut result.kind {
                *len = gen_count as u8;
            }

            let gen_length = Self::read_generic_length(&result.kind, &schema);

            if gen_length != gen_count {
                let mut msg = String::from("Wrong number of generic params. Type requires this many params:");
                msg.push_str(gen_length.to_string().as_str());
                return Err(NP_Error::Custom { message: msg});
            }



            match result.kind.clone() {
                NP_Types::custom { idx } => {
                    match is_rpc {
                        buffer_rpc::none => {
                            if idx < 255 {
                                result.kind = NP_Types::small_custom { idx };
                            }
                        },
                        buffer_rpc::request => {
                            result.kind = NP_Types::rpc_request { idx, func: 0};
                        },
                        buffer_rpc::response => {
                            result.kind = NP_Types::rpc_response { idx, func: 0};
                        }
                    }
                },
                NP_Types::array { len } => {
                    if len < 255 {
                        result.kind = NP_Types::small_array { len: len as u8 };
                    }
                },
                _ => { }
            }

            Ok(Some(result))
        } else { // fast path
            let mut this_type: NP_Types = data_type.into();

            let mut custom_type_idx: Option<usize> = None;

            if let NP_Types::custom { idx } = &mut this_type {
                if let Some(custom_kind) = schema.name_index.get(data_type.trim()) {
                    if let Some(id) = schema.schemas[custom_kind.data].id {
                        *idx = id as usize;
                        custom_type_idx = Some(id as usize);
                    } else {
                        return Err(NP_Error::Custom { message: String::from("Cannot use custom types that don't have an id!")})
                    }
                } else {
                    let mut msg = String::from("Unknown type found!: ");
                    msg.push_str(data_type);
                    return Err(NP_Error::Custom { message: msg });
                }
            }

            let gen_length = Self::read_generic_length(&this_type, &schema);

            // should we have generic params?
            if gen_length != 0 {
                let mut msg = String::from("Generic params required but none provided. Type requires this many params: ");
                msg.push_str(gen_length.to_string().as_str());
                return Err(NP_Error::Custom { message: msg});
            }

            if let Some(idx) = custom_type_idx {
                match is_rpc {
                    buffer_rpc::none => { },
                    buffer_rpc::request => {
                        this_type = NP_Types::rpc_request { idx, func: 0};
                    },
                    buffer_rpc::response => {
                        this_type = NP_Types::rpc_response { idx, func: 0};
                    }
                }
            }

            if None == custom_type_idx && is_rpc != &buffer_rpc::none {
                return Err(NP_Error::Custom { message: String::from("Rpc request did not find impl block!") })
            }

            if let NP_Types::custom { idx } = this_type.clone() {
                if idx < 255 {
                    this_type = NP_Types::small_custom { idx };
                }
            }

            Ok(Some(Self {
                kind: this_type,
                generics: None
            }))
        }
    }
}

#[cfg(test)]
mod schema_tests {
    use crate::schema::NP_Schema;
    use alloc::sync::Arc;
    use crate::error::NP_Error;
    use crate::buffer::type_parser::{NP_Buffer_Type, NP_Types, buffer_rpc};

    fn type_parse_schema() -> Result<Arc<NP_Schema>, NP_Error> {
        let schema = Arc::new(NP_Schema::parse(r##"
            struct myType<X> [id: 10] {
                username: string,
                password: string
            }
            struct anotherType [id: 9] {
                username: string
            }

            struct genericCity<A, B, C, D, E, F, G, H> [id: 11] {
                emaill: string
            }

            struct bigType [id: 500] {
                username: string
            }
        "##)?);

        // println!("{:#?}", schema);

        Ok(schema)
    }

    #[test]
    fn simple_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "myType<u32>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::small_custom { idx: 10 },
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::u32,
                generics: None
            }])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "myType<u32>");

        Ok(())
    }

    #[test]
    fn vec_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Vec<u32>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::vec,
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::u32,
                generics: None
            }])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Vec<u32>");

        Ok(())
    }

    #[test]
    fn crazy_nesting_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Vec<Vec<Vec<Vec<u32>>>>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::vec,
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::vec,
                generics: Some(vec![NP_Buffer_Type {
                    kind: NP_Types::vec,
                    generics: Some(vec![NP_Buffer_Type {
                        kind: NP_Types::vec,
                        generics: Some(vec![NP_Buffer_Type {
                            kind: NP_Types::u32,
                            generics: None
                        }])
                    }])
                }])
            }])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Vec<Vec<Vec<Vec<u32>>>>");

        Ok(())
    }

    #[test]
    fn super_simple_custom_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "anotherType", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::small_custom { idx: 9 },
            generics: None
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "anotherType");

        Ok(())
    }

    #[test]
    fn simple_custom_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "bigType", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::custom { idx: 500 },
            generics: None
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "bigType");

        Ok(())
    }

    #[test]
    fn simple_array_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "[bool; 20]", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::small_array { len: 20 },
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::bool,
                generics: None
            }])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "[bool; 20]");

        Ok(())
    }

    #[test]
    fn large_array_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "[bool; 500]", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::array { len: 500 },
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::bool,
                generics: None
            }])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "[bool; 500]");

        Ok(())
    }

    #[test]
    fn custom_nested_array_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "myType<[bool; 20]>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::small_custom { idx: 10 },
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::small_array { len: 20 },
                generics: Some(vec![NP_Buffer_Type {
                    kind: NP_Types::bool,
                    generics: None
                }])
            }])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "myType<[bool; 20]>");

        Ok(())
    }

    #[test]
    fn crazy_generics_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;


        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "genericCity<u32, i64, bool, u64, string, uuid, ulid, date>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::small_custom { idx: 11 },
            generics: Some(vec![
                NP_Buffer_Type {
                    kind: NP_Types::u32,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::i64,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::bool,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::u64,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::string,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::uuid,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::ulid,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::date,
                    generics: None
                },
            ])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "genericCity<u32, i64, bool, u64, string, uuid, ulid, date>");

        Ok(())
    }

    #[test]
    fn crazy_generics_type_test_2() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "genericCity<u32, i64, myType<[bool; 20]>, u64, string, uuid, ulid, date>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::small_custom { idx: 11 },
            generics: Some(vec![
                NP_Buffer_Type {
                    kind: NP_Types::u32,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::i64,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::small_custom { idx: 10 },
                    generics: Some(vec![NP_Buffer_Type {
                        kind: NP_Types::small_array { len: 20 },
                        generics: Some(vec![NP_Buffer_Type {
                            kind: NP_Types::bool,
                            generics: None
                        }])
                    }])
                },
                NP_Buffer_Type {
                    kind: NP_Types::u64,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::string,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::uuid,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::ulid,
                    generics: None
                },
                NP_Buffer_Type {
                    kind: NP_Types::date,
                    generics: None
                },
            ])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "genericCity<u32, i64, myType<[bool; 20]>, u64, string, uuid, ulid, date>");

        Ok(())
    }



    #[test]
    fn result_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Result<[bool; 20], string>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::result,
            generics: Some(vec![
                NP_Buffer_Type {
                    kind: NP_Types::small_array { len: 20 },
                    generics: Some(vec![NP_Buffer_Type {
                        kind: NP_Types::bool,
                        generics: None
                    }])
                },
                NP_Buffer_Type {
                    kind: NP_Types::string,
                    generics: None
                },
            ])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Result<[bool; 20], string>");

        Ok(())
    }


    #[test]
    fn tuple_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "([bool; 20], string)", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::tuple { len: 2 },
            generics: Some(vec![
                NP_Buffer_Type {
                    kind: NP_Types::small_array { len: 20 },
                    generics: Some(vec![NP_Buffer_Type {
                        kind: NP_Types::bool,
                        generics: None
                    }])
                },
                NP_Buffer_Type {
                    kind: NP_Types::string,
                    generics: None
                },
            ])
        });
        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "([bool; 20], string)");

        Ok(())
    }

    #[test]
    fn complex_nested_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Vec<([bool; 20], string)>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::vec,
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::tuple { len: 2 },
                generics: Some(vec![
                    NP_Buffer_Type {
                        kind: NP_Types::small_array { len: 20 },
                        generics: Some(vec![NP_Buffer_Type {
                            kind: NP_Types::bool,
                            generics: None
                        }])
                    },
                    NP_Buffer_Type {
                        kind: NP_Types::string,
                        generics: None
                    },
                ])
            }])
        });

        let (length, bytes) = &buffer_type.get_bytes()?;
        // println!("{:?}", &bytes[0..(*length as usize)]);
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Vec<([bool; 20], string)>");

        Ok(())
    }

    #[test]
    fn complex_nested_type_test_2() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Vec<([bool; 20], string)>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::vec,
            generics: Some(vec![NP_Buffer_Type {
                kind: NP_Types::tuple { len: 2 },
                generics: Some(vec![
                    NP_Buffer_Type {
                        kind: NP_Types::small_array { len: 20 },
                        generics: Some(vec![NP_Buffer_Type {
                            kind: NP_Types::bool,
                            generics: None
                        }])
                    },
                    NP_Buffer_Type {
                        kind: NP_Types::string,
                        generics: None
                    },
                ])
            }])
        });

        let (length, bytes) = &buffer_type.get_bytes()?;
        // println!("{:?}", &bytes[0..(*length as usize)]);
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Vec<([bool; 20], string)>");

        Ok(())
    }

    #[test]
    fn complex_nested_type_test_3() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Result<([bool; 20], string), string>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::result,
            generics: Some(vec![
                NP_Buffer_Type {
                    kind: NP_Types::tuple { len: 2 },
                    generics: Some(vec![
                        NP_Buffer_Type {
                            kind: NP_Types::small_array { len: 20 },
                            generics: Some(vec![NP_Buffer_Type {
                                kind: NP_Types::bool,
                                generics: None
                            }])
                        },
                        NP_Buffer_Type {
                            kind: NP_Types::string,
                            generics: None
                        },
                    ])
                },
                NP_Buffer_Type {
                    kind: NP_Types::string,
                    generics: None
                }
            ])
        });

        let (length, bytes) = &buffer_type.get_bytes()?;
        // println!("{:?}", &bytes[0..(*length as usize)]);
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Result<([bool; 20], string), string>");

        Ok(())
    }

    #[test]
    fn unit_type_test() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "()", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::tuple { len: 0 },
            generics: None
        });

        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "()");

        Ok(())
    }

    #[test]
    fn unit_type_test_2() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, "Vec<()>", &schema)?)?;
        assert_eq!(buffer_type, NP_Buffer_Type {
            kind: NP_Types::vec,
            generics: Some(vec![
                NP_Buffer_Type {
                    kind: NP_Types::tuple { len: 0 },
                    generics: None
                }
            ])
        });

        let (length, bytes) = &buffer_type.get_bytes()?;
        let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        assert_eq!(buffer_type, from_bytes_type);
        assert_eq!(buffer_type.generate_string(&schema), "Vec<()>");

        Ok(())
    }

    #[test]
    fn rpc_type_test_1() -> Result<(), NP_Error> {
        let schema = type_parse_schema()?;

        let buffer_type = NP_Error::unwrap(NP_Buffer_Type::parse_type_prc(&buffer_rpc::request, "bigType.get", &schema)?)?;
        // assert_eq!(buffer_type, NP_Buffer_Type {
        //     kind: NP_Types::vec,
        //     generics: Some(vec![
        //         NP_Buffer_Type {
        //             kind: NP_Types::tuple { len: 0 },
        //             generics: None
        //         }
        //     ])
        // });
        //
        // let (length, bytes) = &buffer_type.get_bytes()?;
        // let from_bytes_type = NP_Buffer_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
        // assert_eq!(buffer_type, from_bytes_type);
        // assert_eq!(buffer_type.generate_string(&schema), "Vec<()>");

        Ok(())
    }
}

