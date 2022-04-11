use crate::error::NP_Error;
use crate::map::murmurhash3_x86_32;
use crate::map::HASH_SEED;
use crate::schema::args::NP_Schema_Args;
use crate::schema::ast::AST;
use crate::schema::NP_OrderedMap;
use crate::schema::NP_Schem_Kind;
use crate::schema::NP_Schema_Value;
use crate::schema::AST_STR;
use crate::schema::{NP_Schema, NP_Schema_Index};
use crate::types::NP_String_Casing;
use crate::types::NP_Type;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::args::NP_Args;
use super::NP_Parsed_Generics;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum ChildItemParseState {
    Key,
    Colon,
    Value,
    Comma,
    Finished,
}

macro_rules! schema_number {
    ($source: tt, $arguments: tt, $kind: ty, $default: tt, $min: tt, $max: tt) => {
        if let NP_Schema_Args::MAP(args_map) = &$arguments {
            if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("default") {
                if let Ok(value) = data.read($source).parse::<$kind>() {
                    *$default = value;
                }
            }
            if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("min") {
                if let Ok(value) = data.read($source).parse::<$kind>() {
                    *$min = Some(value);
                }
            }
            if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("max") {
                if let Ok(value) = data.read($source).parse::<$kind>() {
                    *$max = Some(value);
                }
            }
        }
    };
}

macro_rules! schema_geo {
    ($source: tt, $arguments: tt, $kind: ty, $default: tt, $deviser: tt) => {
        if let NP_Schema_Args::MAP(args_map) = &$arguments {
            if let Some(NP_Schema_Args::MAP(lat_lng)) = args_map.get("default") {
                if let Some(NP_Schema_Args::NUMBER(lat)) = lat_lng.get("lat") {
                    if let Some(NP_Schema_Args::NUMBER(lng)) = lat_lng.get("lng") {
                        if let Ok(lat_parsed) = lat.read($source).parse::<f64>() {
                            if let Ok(lng_parsed) = lng.read($source).parse::<f64>() {
                                *$default = (
                                    (lat_parsed * $deviser) as $kind,
                                    (lng_parsed * $deviser) as $kind,
                                );
                            }
                        }
                    }
                }
            }
        }
    };
}

macro_rules! schema_bytes_number {
    ($kind: ty, $default: tt, $min: tt, $max: tt, $schema_section: tt) => {
        if *$default == <$kind>::default() && *$min == None && *$max == None {
            $schema_section.extend_from_slice(&[0u8]);
        } else {
            $schema_section.extend_from_slice(&[1u8]);
            $schema_section.extend_from_slice(&$default.to_le_bytes());
            if let Some(x) = $min {
                $schema_section.extend_from_slice(&[1u8]);
                $schema_section.extend_from_slice(&x.to_le_bytes());
            } else {
                $schema_section.extend_from_slice(&[0u8]);
            }
            if let Some(x) = $max {
                $schema_section.extend_from_slice(&[1u8]);
                $schema_section.extend_from_slice(&x.to_le_bytes());
            } else {
                $schema_section.extend_from_slice(&[0u8]);
            }
        }
    };
}

macro_rules! schema_bytes_dec {
    ($exp: tt, $default: tt, $min: tt, $max: tt, $schema_section: tt) => {
        if *$default == 0 && *$min == None && *$max == None {
            $schema_section.extend_from_slice(&[0u8]);
        } else {
            $schema_section.extend_from_slice(&[1u8]);
            $schema_section.extend_from_slice(&$default.to_le_bytes());
            if let Some(x) = $min {
                $schema_section.extend_from_slice(&[1u8]);
                $schema_section.extend_from_slice(&x.to_le_bytes());
            } else {
                $schema_section.extend_from_slice(&[0u8]);
            }
            if let Some(x) = $max {
                $schema_section.extend_from_slice(&[1u8]);
                $schema_section.extend_from_slice(&x.to_le_bytes());
            } else {
                $schema_section.extend_from_slice(&[0u8]);
            }
        }
        $schema_section.extend_from_slice(&$exp.to_le_bytes());
    };
}

#[allow(dead_code)]
impl NP_Schema {
    pub fn get_source_as_str(&self) -> &str {
        unsafe { &core::str::from_utf8_unchecked(&self.source) }
    }

    // pub fn get_schema_info(&self, type_path: &str) -> Option<NP_Schema_Data> {
    //     if let Some(schema) = self.query_schema(type_path) {
    //         Some(NP_Schema_Data {
    //             id: schema.id,
    //             name: if let Some(x) = schema.name {
    //                 Some(x.read_bytes(&self.source))
    //             } else {
    //                 None
    //             },
    //             data_type: {
    //                 match &schema.kind {
    //                     NP_Type::None               => NP_Schema_Data_Types::none,
    //                     NP_Type::Any         { .. } => NP_Schema_Data_Types::any,
    //                     NP_Type::Info               => NP_Schema_Data_Types::info,
    //                     NP_Type::String      { .. } => NP_Schema_Data_Types::string,
    //                     NP_Type::Char        { .. } => NP_Schema_Data_Types::char,
    //                     NP_Type::Int8        { .. } => NP_Schema_Data_Types::i8,
    //                     NP_Type::Int16       { .. } => NP_Schema_Data_Types::i16,
    //                     NP_Type::Int32       { .. } => NP_Schema_Data_Types::i32,
    //                     NP_Type::Int64       { .. } => NP_Schema_Data_Types::i64,
    //                     NP_Type::Uint8       { .. } => NP_Schema_Data_Types::u8,
    //                     NP_Type::Uint16      { .. } => NP_Schema_Data_Types::u16,
    //                     NP_Type::Uint32      { .. } => NP_Schema_Data_Types::u32,
    //                     NP_Type::Uint64      { .. } => NP_Schema_Data_Types::u64,
    //                     NP_Type::f32         { .. } => NP_Schema_Data_Types::f32,
    //                     NP_Type::f64         { .. } => NP_Schema_Data_Types::f64,
    //                     NP_Type::Dec32       { .. } => NP_Schema_Data_Types::dec32,
    //                     NP_Type::Dec64       { .. } => NP_Schema_Data_Types::dec64,
    //                     NP_Type::Boolean     { .. } => NP_Schema_Data_Types::bool,
    //                     NP_Type::Geo32       { .. } => NP_Schema_Data_Types::geo32,
    //                     NP_Type::Geo64       { .. } => NP_Schema_Data_Types::geo64,
    //                     NP_Type::Geo128      { .. } => NP_Schema_Data_Types::geo128,
    //                     NP_Type::Uuid        { .. } => NP_Schema_Data_Types::uuid,
    //                     NP_Type::Ulid        { .. } => NP_Schema_Data_Types::ulid,
    //                     NP_Type::Date        { .. } => NP_Schema_Data_Types::date,
    //                     NP_Type::Enum        { .. } => NP_Schema_Data_Types::_enum,
    //                     NP_Type::Struct      { .. } => NP_Schema_Data_Types::_struct,
    //                     NP_Type::Map         { .. } => NP_Schema_Data_Types::map,
    //                     NP_Type::Vec         { .. } => NP_Schema_Data_Types::vec,
    //                     NP_Type::Result      { .. } => NP_Schema_Data_Types::result,
    //                     NP_Type::Option      { .. } => NP_Schema_Data_Types::option,
    //                     NP_Type::Array       { .. } => NP_Schema_Data_Types::array,
    //                     NP_Type::Tuple       { .. } => NP_Schema_Data_Types::tuple,
    //                     NP_Type::Impl        { .. } => NP_Schema_Data_Types::_impl,
    //                     NP_Type::Fn_Self     { .. } => NP_Schema_Data_Types::_self,
    //                     NP_Type::Method      { .. } => NP_Schema_Data_Types::_fn,
    //                     NP_Type::Generic     { .. } => NP_Schema_Data_Types::generic,
    //                     NP_Type::Custom      { .. } => NP_Schema_Data_Types::custom,
    //                     NP_Type::Box         { .. } => NP_Schema_Data_Types::_box,
    //                     NP_Type::Simple_Enum { .. } => NP_Schema_Data_Types::_enum,
    //                 }
    //             },
    //             generics: match &schema.generics {
    //                 NP_Parsed_Generics::None => None,
    //                 NP_Parsed_Generics::Arguments(_idx, args) => Some(args.len()),
    //                 NP_Parsed_Generics::Types(types) => Some(types.len())
    //             },
    //             has_args: if let NP_Schema_Args::NULL = &schema.arguments {
    //                 false
    //             } else {
    //                 true
    //             }
    //         })
    //     } else {
    //         None
    //     }
    // }

    // pub fn query_schema(&self, type_path: &str) -> Option<&NP_Type> {

    //     if self.schemas.len() == 0 {
    //         return None;
    //     }

    //     let dot_pos = type_path.chars().enumerate().fold(None, |accu, (i, elem)| {
    //         if accu == None {
    //             if elem == '.' {
    //                 Some(i)
    //             } else {
    //                 None
    //             }
    //         } else {
    //             accu
    //         }
    //     });

    //     #[derive(PartialEq, Debug)]
    //     enum scan_state {
    //         query,
    //         last_pass,
    //         completed
    //     }

    //     let type_name_ref = type_path;
    //     let type_name_chars = type_path.as_bytes();

    //     let mut state = scan_state::query;

    //     if let Some(first_dot) = dot_pos { // nested type

    //         let mut level: usize = 0;

    //         let mut current_idx = (0, first_dot);
    //         let mut check_path = type_name_ref;
    //         let mut use_schema = &self.schemas[0];

    //         while state != scan_state::completed {

    //             check_path = &type_name_ref[current_idx.0..current_idx.1];
    //             current_idx.0 = current_idx.1 + 1;
    //             current_idx.1 += 1;

    //             if level == 0 {
    //                 if let Some(info) = self.name_index.get(check_path) {
    //                     use_schema = &self.schemas[info.data];
    //                 } else {
    //                     return None;
    //                 }
    //             } else {

    //                 if check_path == "_generics" {

    //                     while current_idx.1 < type_name_ref.len() && type_name_chars[current_idx.1] != '.' as u8 {
    //                         current_idx.1 += 1;
    //                     }
    //                     check_path = &type_name_ref[current_idx.0..current_idx.1];

    //                     if let NP_Parsed_Generics::Types(types) =  &use_schema.generics {
    //                         if let Ok(indx) = check_path.parse::<usize>() {
    //                             use_schema = &self.schemas[types[indx]];

    //                             if current_idx.1 == type_name_ref.len() {
    //                                 state = scan_state::last_pass;
    //                             } else {
    //                                 current_idx.0 = current_idx.1 + 1;
    //                                 current_idx.1 += 1;
    //                             }

    //                         } else {
    //                             return None;
    //                         }
    //                     } else {
    //                         return None;
    //                     }
    //                 } else {
    //                     match &use_schema.kind {
    //                         NP_Type::Enum { children, .. } => {
    //                             if let Some(x) = children.get(check_path) {
    //                                 if let Some(child_type) = x {
    //                                     use_schema = &self.schemas[*child_type];
    //                                 } else {
    //                                     return None;
    //                                 }
    //                             } else {
    //                                 return None;
    //                             }
    //                         }
    //                         NP_Type::Struct { children, .. } => {
    //                             if let Some(x) = children.get(check_path) {
    //                                 use_schema = &self.schemas[*x];
    //                             } else {
    //                                 return None;
    //                             }
    //                         }
    //                         NP_Type::Tuple { children, .. } => {
    //                             if let Ok(idx) = check_path.parse::<usize>() {
    //                                 use_schema = &self.schemas[idx];
    //                             } else {
    //                                 return None;
    //                             }
    //                         },
    //                         _ => {
    //                             return None;
    //                         }
    //                     }
    //                 }
    //             }

    //             if state == scan_state::last_pass {
    //                 return Some(use_schema);
    //             }

    //             while current_idx.1 < type_name_ref.len() && type_name_chars[current_idx.1] != '.' as u8 {
    //                 current_idx.1 += 1;
    //             }

    //             if state == scan_state::last_pass {
    //                 state = scan_state::completed;
    //             }

    //             if current_idx.1 == type_name_ref.len() {
    //                 state = scan_state::last_pass;
    //             }

    //             level += 1;
    //         }

    //         None
    //     } else { // base type
    //         if let Some(info) = self.name_index.get(type_path.as_ref()) {
    //             Some(&self.schemas[info.data])
    //         } else {
    //             None
    //         }
    //     }
    // }

    pub fn parse<S>(input: S) -> Result<Self, NP_Error>
    where
        S: AsRef<str>,
    {
        let ast = AST::parse(input.as_ref())?;

        let mut parse_idx: usize = 0;
        let mut parse_schema: Vec<NP_Schema_Value> = Vec::new();
        let mut type_idx: NP_OrderedMap<NP_Schema_Index> = NP_OrderedMap::new();

        let top_generics = NP_Parsed_Generics::None;

        let mut max_loop: u32 = 0;

        while parse_idx < ast.len() && max_loop < (u32::MAX / 2) {
            max_loop += 1;

            if ast[parse_idx] == AST::newline || ast[parse_idx] == AST::semicolon {
                parse_idx += 1;
            } else {
                parse_idx = Self::parse_single_type(
                    input.as_ref(),
                    &ast,
                    parse_idx,
                    0,
                    0,
                    &top_generics,
                    &mut type_idx,
                    &mut parse_schema,
                )?;
                parse_idx += 1;
            }
        }

        // build ID index
        let mut max_id: usize = 0;
        for schema in &parse_schema {
            if let Some(id) = schema.id {
                max_id = usize::max(id, max_id);
            }
        }

        max_id += 1;

        let mut id_idx: Vec<NP_Schema_Index> = if parse_schema.len() == 0 {
            vec![]
        } else {
            vec![NP_Schema_Index::default(); max_id as usize]
        };

        for schema in &parse_schema {
            if let Some(id) = schema.id {
                if let Some(name) = schema.name {
                    if let Some(schema_index) = type_idx.get(name.read(input.as_ref())) {
                        id_idx[id as usize] = schema_index.clone();
                    }
                }
            }
        }

        // calculate unique id for this schema based on info
        let mut unique_id: u32 = 0;

        if let Some(info) = type_idx.get("__info") {
            let info_schema = &parse_schema[info.data];
            if let Some(id) = info_schema.args.query("id", input.as_ref()) {
                match id {
                    NP_Args::NUMBER(num) => {
                        let hash = murmurhash3_x86_32(num.as_bytes(), HASH_SEED);
                        unique_id = unique_id.wrapping_add(hash);
                    }
                    NP_Args::STRING(stri) => {
                        let hash = murmurhash3_x86_32(stri.as_bytes(), HASH_SEED);
                        unique_id = unique_id.wrapping_add(hash);
                    }
                    _ => {}
                }
            }
            if let Some(id) = info_schema.args.query("version", input.as_ref()) {
                match id {
                    NP_Args::NUMBER(num) => {
                        let hash = murmurhash3_x86_32(num.as_bytes(), HASH_SEED);
                        unique_id = unique_id.wrapping_add(hash);
                    }
                    NP_Args::STRING(stri) => {
                        let hash = murmurhash3_x86_32(stri.as_bytes(), HASH_SEED);
                        unique_id = unique_id.wrapping_add(hash);
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            source: String::from(input.as_ref()).into_bytes(),
            schemas: parse_schema,
            name_index: type_idx,
            id_index: id_idx,
            unique_id: unique_id,
        })
    }

    fn maybe_error_on_generics(result_schema: &NP_Schema_Value) -> Result<(), NP_Error> {
        if let NP_Parsed_Generics::Parent(_, _) = &result_schema.generics {
            match &result_schema.kind.val {
                NP_Type::Enum { .. } => {}
                NP_Type::Struct { .. } => {}
                NP_Type::Tuple { .. } => {}
                NP_Type::Impl { .. } => {}
                NP_Type::Custom { .. } => {}
                _ => {
                    let mut msg =
                        String::from("Error: this type does not support generic arguments: ");
                    msg.push_str(result_schema.kind.val.get_str());
                    return Err(NP_Error::Custom { message: msg });
                } // NP_Type::Generic { .. } => {}
            }
        }

        Ok(())
    }

    fn maybe_parse_children(
        ast: &Vec<AST>,
        index: usize,
        max_index: usize,
        is_tuple: bool,
    ) -> (usize, Option<&Vec<AST>>) {
        if index + 1 >= max_index {
            return (index, None);
        }

        if is_tuple {
            match &ast[index + 1] {
                AST::parans { items } => (index + 1, Some(items)),
                _ => (index, None),
            }
        } else {
            match &ast[index + 1] {
                AST::curly { items } => (index + 1, Some(items)),
                _ => (index, None),
            }
        }
    }

    fn maybe_parse_title(
        ast: &Vec<AST>,
        index: usize,
        max_index: usize,
        result_schema: &mut NP_Schema_Value,
    ) -> usize {
        if index + 1 >= max_index {
            return index;
        }

        match &ast[index + 1] {
            AST::token { addr } => {
                result_schema.name = Some(addr.clone());
                index + 1
            }
            _ => index,
        }
    }

    fn maybe_parse_generics(
        ast: &Vec<AST>,
        index: usize,
        max_index: usize,
        schema_len: usize,
        result_schema: &mut NP_Schema_Value,
    ) -> Result<usize, NP_Error> {
        if index + 1 >= max_index {
            return Ok(index);
        }

        match &ast[index + 1] {
            AST::xml { items } => {
                let mut generics: Vec<AST_STR> = Vec::new();

                for generic_item in items.iter() {
                    match generic_item {
                        AST::token { addr } => generics.push(addr.clone()),
                        AST::comma => {}
                        AST::newline => {}
                        _ => {
                            return Err(NP_Error::Custom {
                                message: String::from("Unexpected token in generics!"),
                            })
                        }
                    }
                }

                if result_schema.generics != NP_Parsed_Generics::None {
                    return Err(NP_Error::Custom { message: String::from("Attempting to use generic arguments on a type that already has generic types!") });
                }

                result_schema.generics = NP_Parsed_Generics::Parent(schema_len, generics);

                Ok(index + 1)
            }
            _ => Ok(index),
        }
    }

    fn parse_argument_groups(source: &str, items: &Vec<AST>) -> Result<NP_Schema_Args, NP_Error> {
        let mut has_colons = false;

        for item in items {
            if *item == AST::colon {
                has_colons = true;
            }
        }

        let mut i = 0;

        if has_colons {
            // key: value, key: value

            let mut state = ChildItemParseState::Key;

            let mut key_str: AST_STR = Default::default();
            let mut final_args = NP_OrderedMap::new();
            while i < items.len() && state != ChildItemParseState::Finished {
                match state {
                    ChildItemParseState::Key => {
                        if let AST::token { addr } = items[i] {
                            key_str = addr.clone();
                            state = ChildItemParseState::Colon;
                            i += 1;
                        } else {
                            return Err(NP_Error::Custom {
                                message: String::from("Error parsing argument key:value pairs!"),
                            });
                        }
                    }
                    ChildItemParseState::Colon => {
                        // colon
                        if items[i] != AST::colon {
                            return Err(NP_Error::Custom {
                                message: String::from("Error parsing argument key:value pairs!"),
                            });
                        } else {
                            state = ChildItemParseState::Value;
                            i += 1;
                        }
                    }
                    ChildItemParseState::Value => {
                        // value

                        match &items[i] {
                            AST::token { addr } => {
                                let token_value = addr.read(source);
                                match token_value {
                                    "true" => {
                                        final_args.set(key_str.read(source), NP_Schema_Args::TRUE);
                                    }
                                    "false" => {
                                        final_args.set(key_str.read(source), NP_Schema_Args::FALSE);
                                    }
                                    "null" => {
                                        final_args.set(key_str.read(source), NP_Schema_Args::NULL);
                                    }
                                    _ => {}
                                }
                            }
                            AST::number { addr } => {
                                final_args.set(
                                    key_str.read(source),
                                    NP_Schema_Args::NUMBER(addr.clone()),
                                );
                            }
                            AST::string { addr } => {
                                final_args.set(
                                    key_str.read(source),
                                    NP_Schema_Args::STRING(addr.clone()),
                                );
                            }
                            AST::square { items } => {
                                final_args.set(
                                    key_str.read(source),
                                    Self::parse_argument_groups(source, items)?,
                                );
                            }
                            _ => {
                                return Err(NP_Error::Custom {
                                    message: String::from(
                                        "Error parsing argument key:value pairs!",
                                    ),
                                })
                            }
                        }

                        state = ChildItemParseState::Comma;
                        i += 1;
                    }
                    ChildItemParseState::Comma => {
                        // comma
                        while i < items.len()
                            && (&items[i] == &AST::comma || &items[i] == &AST::newline)
                        {
                            i += 1;
                        }
                        state = ChildItemParseState::Key;
                    }
                    _ => {} // other
                }
            }

            Ok(NP_Schema_Args::MAP(final_args))
        } else {
            // value, value, value

            let mut final_args = Vec::new();
            let mut state = ChildItemParseState::Key;

            let mut i = 0;
            while i < items.len() && state != ChildItemParseState::Finished {
                match state {
                    ChildItemParseState::Key => {
                        match &items[i] {
                            AST::token { addr } => {
                                let token_value = addr.read(source);
                                match token_value {
                                    "true" => {
                                        final_args.push(NP_Schema_Args::TRUE);
                                    }
                                    "false" => {
                                        final_args.push(NP_Schema_Args::FALSE);
                                    }
                                    "null" => {
                                        final_args.push(NP_Schema_Args::NULL);
                                    }
                                    _ => {}
                                }
                            }
                            AST::number { addr } => {
                                final_args.push(NP_Schema_Args::NUMBER(addr.clone()));
                            }
                            AST::string { addr } => {
                                final_args.push(NP_Schema_Args::STRING(addr.clone()));
                            }
                            AST::square { items } => {
                                final_args.push(Self::parse_argument_groups(source, items)?);
                            }
                            _ => {
                                return Err(NP_Error::Custom {
                                    message: String::from(
                                        "Error parsing argument key:value pairs!",
                                    ),
                                })
                            }
                        }

                        state = ChildItemParseState::Comma;
                        i += 1;
                    }
                    ChildItemParseState::Comma => {
                        while i < items.len()
                            && (&items[i] == &AST::comma || &items[i] == &AST::newline)
                        {
                            i += 1;
                        }
                        state = ChildItemParseState::Key;
                    }
                    _ => {}
                }
            }

            Ok(NP_Schema_Args::LIST(final_args))
        }
    }

    fn maybe_parse_arguments(
        source: &str,
        ast: &Vec<AST>,
        index: usize,
        max_index: usize,
        result_schema: &mut NP_Schema_Value,
    ) -> Result<usize, NP_Error> {
        if index + 1 >= max_index {
            return Ok(index);
        }

        match &ast[index + 1] {
            AST::square { items } => {
                result_schema.args = Self::parse_argument_groups(source, items)?;
                Ok(index + 1)
            }
            _ => Ok(index),
        }
    }

    fn str_to_type(
        source: &str,
        token: &AST_STR,
        parent_generics: &NP_Parsed_Generics,
        type_idx: &NP_OrderedMap<NP_Schema_Index>,
    ) -> Option<NP_Type<usize, AST_STR>> {
        let token_value = token.read(source);

        match token_value {
            "any" => Some(NP_Type::Any),
            "info" => Some(NP_Type::Info),
            "string" => Some(NP_Type::String {
                default: Default::default(),
                casing: Default::default(),
                max_len: Default::default(),
            }),
            "char" => Some(NP_Type::Char {
                default: Default::default(),
            }),
            "i8" => Some(NP_Type::Int8 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "i16" => Some(NP_Type::Int16 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "i32" => Some(NP_Type::Int32 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "i64" => Some(NP_Type::Int64 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "u8" => Some(NP_Type::Uint8 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "u16" => Some(NP_Type::Uint16 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "u32" => Some(NP_Type::Uint32 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "u64" => Some(NP_Type::Uint64 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "f32" => Some(NP_Type::Float32 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "f64" => Some(NP_Type::Float64 {
                default: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "e32" => Some(NP_Type::Exp32 {
                default: Default::default(),
                e: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "e64" => Some(NP_Type::Exp64 {
                default: Default::default(),
                e: Default::default(),
                min: Default::default(),
                max: Default::default(),
            }),
            "bool" => Some(NP_Type::Bool {
                default: Default::default(),
            }),
            "g32" => Some(NP_Type::Geo32 {
                default: Default::default(),
            }),
            "g64" => Some(NP_Type::Geo64 {
                default: Default::default(),
            }),
            "g128" => Some(NP_Type::Geo128 {
                default: Default::default(),
            }),
            "uuid" => Some(NP_Type::Uuid),
            "ulid" => Some(NP_Type::Ulid),
            "date" => Some(NP_Type::Date {
                default: Default::default(),
            }),
            "enum" => Some(NP_Type::Enum {
                children: Default::default(),
                default: Default::default(),
            }),
            "struct" => Some(NP_Type::Struct {
                children: Default::default(),
            }),
            "Map" => Some(NP_Type::Map {
                of: Default::default(),
            }),
            "Vec" => Some(NP_Type::Vec {
                of: Default::default(),
                max_len: Default::default(),
            }),
            "List" => Some(NP_Type::List {
                of: Default::default(),
            }),
            "Result" => Some(NP_Type::Result {
                ok: Default::default(),
                err: Default::default(),
            }),
            "Option" => Some(NP_Type::Option {
                some: Default::default(),
            }),
            "Box" => Some(NP_Type::Box {
                of: Default::default(),
            }),
            "impl" => Some(NP_Type::Impl {
                methods: Default::default(),
            }),
            "self" => Some(NP_Type::This {
                parent_schema_addr: Default::default(),
            }),
            "Self" => Some(NP_Type::This {
                parent_schema_addr: Default::default(),
            }),
            "tuple" => Some(NP_Type::Tuple {
                children: Default::default(),
            }),
            _ => {
                if let NP_Parsed_Generics::Parent(parent_idx, these_generics) = parent_generics {
                    for (idx, generic_ast) in these_generics.iter().enumerate() {
                        if generic_ast.read(source) == token_value {
                            return Some(NP_Type::Generic {
                                parent_schema_addr: *parent_idx,
                                parent_generic_idx: idx,
                            });
                        }
                    }
                }

                // is this a valid custom type?
                if let Some(type_data) = type_idx.get(token_value) {
                    return Some(NP_Type::Custom {
                        parent_schema_addr: type_data.data,
                        generic_args: None,
                    });
                }

                return None;
            }
        }
    }

    fn parse_single_type(
        source: &str,
        ast: &Vec<AST>,
        index: usize,
        depth: u16,
        parent_idx: usize,
        generics: &NP_Parsed_Generics,
        type_idx: &mut NP_OrderedMap<NP_Schema_Index>,
        parsed_schema: &mut Vec<NP_Schema_Value>,
    ) -> Result<usize, NP_Error> {
        if depth > 255 {
            return Err(NP_Error::RecursionLimit);
        }

        // find where the next newline, semicolon or comma is.  Parsing should not pass this point.
        let mut max_index = index;
        while max_index < ast.len()
            && ast[max_index] != AST::semicolon
            && ast[max_index] != AST::newline
            && ast[max_index] != AST::comma
        {
            max_index += 1;
        }

        let mut use_index = index;
        let this_ast = &ast[use_index];
        let mut result_schema: NP_Schema_Value = Default::default();
        // inject placeholder schema
        let this_schema_addr = parsed_schema.len();
        parsed_schema.push(Default::default());

        let mut internal_type_args: Vec<usize> = Vec::new();

        let mut is_implicit = false;
        let mut is_struct = false;

        let mut contents_of_type = match this_ast {
            AST::curly { items } => {
                // implicit struct { key: X }
                result_schema.kind = NP_Schem_Kind::new(NP_Type::Struct {
                    children: Default::default(),
                });
                is_implicit = true;
                is_struct = true;
                Some(items)
            }
            AST::parans { items } => {
                // tuple type (X, Y, Z) or method (x, y) -> z
                let mut has_arrows = false;
                let mut check_index = use_index;
                while check_index < max_index {
                    if let AST::arrow = &ast[check_index] {
                        has_arrows = true;
                    }
                    check_index += 1;
                }

                if has_arrows {
                    result_schema.kind = NP_Schem_Kind::new(NP_Type::Method {
                        id: Default::default(),
                        args: Default::default(),
                        returns: Default::default(),
                    });
                } else {
                    is_implicit = true;
                    result_schema.kind = NP_Schem_Kind::new(NP_Type::Tuple {
                        children: Default::default(),
                    });
                    use_index =
                        Self::maybe_parse_title(ast, use_index, max_index, &mut result_schema);
                    use_index = Self::maybe_parse_arguments(
                        source,
                        ast,
                        use_index,
                        max_index,
                        &mut result_schema,
                    )?;
                }

                Some(items)
            }
            AST::square { items } => {
                // array type [X; 32]
                result_schema.kind = NP_Schem_Kind::new(NP_Type::Array {
                    of: Default::default(),
                    len: Default::default(),
                });
                use_index = Self::maybe_parse_title(ast, use_index, max_index, &mut result_schema);
                use_index = Self::maybe_parse_arguments(
                    source,
                    ast,
                    use_index,
                    max_index,
                    &mut result_schema,
                )?;
                Some(items)
            }
            AST::token { addr } => {
                // standard named type

                // handle types with generic parameters like Vec<u32> or List<X, Y, Z>
                if ast.len() > use_index + 1 {
                    if let AST::xml { items } = &ast[use_index + 1] {
                        if addr.read(source) != "impl" {
                            // ignore impls
                            let mut i: usize = 0;
                            while i < usize::min(items.len(), 24) {
                                if items[i] != AST::comma && items[i] != AST::newline {
                                    internal_type_args.push(parsed_schema.len());
                                    i = Self::parse_single_type(
                                        &source,
                                        items,
                                        i,
                                        depth + 1,
                                        parent_idx,
                                        &generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                }
                                i += 1;
                            }
                        }

                        use_index += 1;
                    }
                }

                if internal_type_args.len() > 0 {
                    // result_schema.generics = NP_Parsed_Generics::Types(child_generics);
                    // result_schema.use_generics = Some(child_generics);
                }

                use_index = Self::maybe_parse_title(ast, use_index, max_index, &mut result_schema);
                use_index = Self::maybe_parse_generics(
                    ast,
                    use_index,
                    max_index,
                    this_schema_addr,
                    &mut result_schema,
                )?;
                use_index = Self::maybe_parse_arguments(
                    source,
                    ast,
                    use_index,
                    max_index,
                    &mut result_schema,
                )?;

                if let Some(data_type) = Self::str_to_type(source, addr, &generics, &type_idx) {
                    result_schema.kind = NP_Schem_Kind::new(data_type);
                    if let NP_Type::Struct { .. } = &result_schema.kind.val {
                        is_struct = true;
                    }
                } else {
                    // no type found!
                    let mut err = String::from("Unknown type found!: ");
                    err.push_str(addr.read(source));
                    return Err(NP_Error::Custom { message: err });
                }

                None
            }
            _ => {
                return Err(NP_Error::Custom {
                    message: String::from("Unexpected value in parsing AST!"),
                })
            }
        };

        // set type index
        if let Some(title) = result_schema.name {
            if depth == 0 {
                if let NP_Type::Impl { .. } = result_schema.kind.val {
                    // impl block

                    let index_data = if let Some(index_data) = type_idx.get(title.read(source)) {
                        index_data.clone()
                    } else {
                        return Err(NP_Error::Custom {
                            message: String::from("impl block before data declaration!"),
                        });
                    };

                    type_idx.set(
                        title.read(source),
                        NP_Schema_Index {
                            data: index_data.data,
                            methods: Some(this_schema_addr),
                        },
                    );
                } else {
                    // any other type
                    type_idx.set(
                        title.read(source),
                        NP_Schema_Index {
                            data: this_schema_addr,
                            methods: None,
                        },
                    );
                }
            }
        }

        // handle this condition:
        // struct (/* really a tuple */)
        if is_struct && !is_implicit && max_index > use_index + 1 {
            match &ast[use_index + 1] {
                AST::parans { .. } => {
                    // actually a tuple type!
                    result_schema.kind = NP_Schem_Kind::new(NP_Type::Tuple {
                        children: Vec::new(),
                    });
                }
                _ => {}
            }
        }

        // type generics not allowed on nested types
        if let NP_Parsed_Generics::Parent(_, _) = &result_schema.generics {
            if depth > 0 {
                return Err(NP_Error::Custom {
                    message: String::from("Nested types cannot have generic arguments!"),
                });
            }
        }

        let mut enum_keys: Vec<AST_STR> = Vec::new();

        match &mut result_schema.kind.val {
            NP_Type::None => { /* nothing to do */ }
            NP_Type::Any => { /* nothing to do */ }
            NP_Type::Info => { /* nothing to do */ }
            NP_Type::String {
                default,
                casing,
                max_len,
                ..
            } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::STRING(data)) = args_map.get("default") {
                        *default = data.clone();
                    }
                    if let Some(NP_Schema_Args::TRUE) = args_map.get("uppercase") {
                        *casing = NP_String_Casing::Uppercase;
                    }
                    if let Some(NP_Schema_Args::TRUE) = args_map.get("lowercase") {
                        *casing = NP_String_Casing::Lowercase;
                    }
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("max_len") {
                        if let Ok(length) = data.read(source).parse::<usize>() {
                            *max_len = Some(length);
                        }
                    }
                }
            }
            NP_Type::Char { default, .. } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::STRING(data)) = args_map.get("default") {
                        if let Some(char) = data.read(source).chars().next() {
                            *default = char;
                        }
                    }
                }
            }
            NP_Type::Int8 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, i8, default, min, max);
            }
            NP_Type::Int16 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, i16, default, min, max);
            }
            NP_Type::Int32 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, i32, default, min, max);
            }
            NP_Type::Int64 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, i64, default, min, max);
            }
            NP_Type::Uint8 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, u8, default, min, max);
            }
            NP_Type::Uint16 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, u16, default, min, max);
            }
            NP_Type::Uint32 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, u32, default, min, max);
            }
            NP_Type::Uint64 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, u64, default, min, max);
            }
            NP_Type::Float32 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, f32, default, min, max);
            }
            NP_Type::Float64 {
                default, min, max, ..
            } => {
                let args = &result_schema.args;
                schema_number!(source, args, f64, default, min, max);
            }
            NP_Type::Exp32 {
                default,
                e,
                min,
                max,
                ..
            } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("exp") {
                        if let Ok(value) = data.read(source).parse::<i8>() {
                            *e = value;
                        }
                    }

                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("default") {
                        if let Ok(value) = data.read(source).parse::<i32>() {
                            *default = value;
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("min") {
                        if let Ok(value) = data.read(source).parse::<i32>() {
                            *min = Some(value);
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("max") {
                        if let Ok(value) = data.read(source).parse::<i32>() {
                            *max = Some(value);
                        }
                    }
                }
            }
            NP_Type::Exp64 {
                default,
                e,
                min,
                max,
                ..
            } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("exp") {
                        if let Ok(value) = data.read(source).parse::<i16>() {
                            *e = value;
                        }
                    }

                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("default") {
                        if let Ok(value) = data.read(source).parse::<i64>() {
                            *default = value;
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("min") {
                        if let Ok(value) = data.read(source).parse::<i64>() {
                            *min = Some(value);
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("max") {
                        if let Ok(value) = data.read(source).parse::<i64>() {
                            *max = Some(value);
                        }
                    }
                }
            }
            NP_Type::Bool { default, .. } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::TRUE) = args_map.get("default") {
                        *default = true;
                    }
                    if let Some(NP_Schema_Args::FALSE) = args_map.get("default") {
                        *default = false;
                    }
                }
            }
            NP_Type::Geo32 { default, .. } => {
                let args = &result_schema.args;
                schema_geo!(source, args, i16, default, 100f64);
            }
            NP_Type::Geo64 { default, .. } => {
                let args = &result_schema.args;
                schema_geo!(source, args, i32, default, 10000000f64);
            }
            NP_Type::Geo128 { default, .. } => {
                let args = &result_schema.args;
                schema_geo!(source, args, i64, default, 1000000000f64);
            }
            NP_Type::Uuid { .. } => {}
            NP_Type::Ulid { .. } => {}
            NP_Type::Date { default, .. } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("default") {
                        if let Ok(value) = data.read(source).parse::<u64>() {
                            *default = value;
                        }
                    }
                }
            }
            NP_Type::Enum { children, default } => {
                let (next_index, children_items_ast) =
                    Self::maybe_parse_children(ast, use_index, max_index, false);
                use_index = next_index;
                contents_of_type = children_items_ast;

                if let Some(children_ast) = contents_of_type {
                    let mut parse_idx: usize = 0;
                    let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Key;

                    while parse_idx < children_ast.len() {
                        match parse_state {
                            ChildItemParseState::Key => {
                                if let AST::token { addr } = &children_ast[parse_idx] {
                                    key_ast = addr.clone();

                                    if parse_idx + 1 >= children_ast.len() {
                                        enum_keys.push(key_ast.clone());
                                        children.set(key_ast.read(source), None);
                                        parse_state = ChildItemParseState::Finished;
                                        parse_idx += 1;
                                    } else {
                                        parse_state = ChildItemParseState::Colon;
                                        parse_idx += 1;
                                    }
                                } else {
                                    return Err(NP_Error::Custom {
                                        message: String::from("Error parsing enum child items!"),
                                    });
                                }
                            }
                            ChildItemParseState::Colon => {
                                match &children_ast[parse_idx] {
                                    AST::comma => {
                                        // has no child types
                                        children.set(key_ast.read(source), None);
                                        parse_state = ChildItemParseState::Comma;
                                        parse_idx += 1;
                                    }
                                    AST::parans { .. } => {
                                        parse_state = ChildItemParseState::Value;
                                    }
                                    AST::curly { .. } => {
                                        parse_state = ChildItemParseState::Value;
                                    }
                                    AST::newline => {
                                        // has no child types
                                        children.set(key_ast.read(source), None);
                                        parse_state = ChildItemParseState::Comma;
                                        parse_idx += 1;
                                    }
                                    _ => {
                                        return Err(NP_Error::Custom {
                                            message: String::from(
                                                "Error parsing enum child items!",
                                            ),
                                        });
                                    }
                                }
                            }
                            ChildItemParseState::Value => {
                                let schema_loc = parsed_schema.len();
                                children.set(key_ast.read(source), Some(schema_loc));

                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        this_schema_addr,
                                        &result_schema.generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                } else {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        parent_idx,
                                        &generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            }
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            }
                                            AST::newline => {
                                                parse_idx += 1;
                                            }
                                            _ => {
                                                parse_state = ChildItemParseState::Key;
                                            }
                                        }
                                    } else {
                                        parse_state = ChildItemParseState::Finished
                                    }

                                    loop_max += 1; // prevent infinite loop
                                    if loop_max == u8::MAX {
                                        parse_state = ChildItemParseState::Finished
                                    }
                                }
                            }
                            ChildItemParseState::Finished => {
                                // nothing to do here
                            }
                        }
                    }

                    let mut default_key: Option<String> = None;

                    if let NP_Schema_Args::MAP(data) = &result_schema.args {
                        if let Some(NP_Schema_Args::STRING(data)) = data.get("default") {
                            for (idx, key) in children.iter_keys().enumerate() {
                                if key == data.read(source) {
                                    *default = idx;
                                    default_key = Some(key.clone());
                                }
                            }
                        }
                    }

                    if let Some(key) = default_key {
                        if let Some(default_type) = children.get(key.as_str()) {
                            if let Some(_child_type) = default_type {
                                return Err(NP_Error::Custom {
                                    message: String::from(
                                        "Enum default cannot contain properties!",
                                    ),
                                });
                            }
                        }
                    }
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Missing enum children declaration!"),
                    });
                }
            }
            NP_Type::Struct { children } => {
                if is_implicit == false {
                    let (next_index, children_ast_items) =
                        Self::maybe_parse_children(ast, use_index, max_index, false);
                    use_index = next_index;
                    contents_of_type = children_ast_items;
                }

                if let Some(children_ast) = contents_of_type {
                    let mut parse_idx: usize = 0;
                    let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Key;

                    while parse_idx < children_ast.len() {
                        match parse_state {
                            ChildItemParseState::Key => {
                                if let AST::token { addr } = &children_ast[parse_idx] {
                                    key_ast = addr.clone();
                                    parse_state = ChildItemParseState::Colon;
                                    parse_idx += 1;
                                } else {
                                    return Err(NP_Error::Custom {
                                        message: String::from("Error parsing struct child items!"),
                                    });
                                }
                            }
                            ChildItemParseState::Colon => {
                                if let AST::colon = &children_ast[parse_idx] {
                                    parse_state = ChildItemParseState::Value;
                                    parse_idx += 1;
                                } else {
                                    return Err(NP_Error::Custom {
                                        message: String::from("Error parsing struct child items!"),
                                    });
                                }
                            }
                            ChildItemParseState::Value => {
                                let schema_loc = parsed_schema.len();
                                children.set(key_ast.read(source), schema_loc);
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        this_schema_addr,
                                        &result_schema.generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                } else {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        parent_idx,
                                        &generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            }
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            }
                                            AST::newline => {
                                                parse_idx += 1;
                                            }
                                            _ => {
                                                parse_state = ChildItemParseState::Key;
                                            }
                                        }
                                    } else {
                                        parse_state = ChildItemParseState::Finished
                                    }

                                    loop_max += 1; // prevent infinite loop
                                    if loop_max == u8::MAX {
                                        parse_state = ChildItemParseState::Finished
                                    }
                                }
                            }
                            ChildItemParseState::Finished => {
                                // nothing to do here
                            }
                        }
                    }
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Missing struct children declaration!"),
                    });
                }
            }
            NP_Type::Map { of, .. } => {
                if internal_type_args.len() == 1 {
                    *of = Box::new(internal_type_args[0]);
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Maps require one argument for contents: Map<X>"),
                    });
                }
            }
            NP_Type::List { of, .. } => {
                if internal_type_args.len() == 1 {
                    *of = Box::new(internal_type_args[0]);
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from(
                            "Linked lists require one argument for contents: List<X>",
                        ),
                    });
                }
            }
            NP_Type::Vec { max_len, .. } => {
                if let NP_Schema_Args::MAP(args_map) = &result_schema.args {
                    if let Some(NP_Schema_Args::NUMBER(data)) = args_map.get("max_len") {
                        if let Ok(value) = data.read(source).parse::<usize>() {
                            *max_len = Some(value);
                        }
                    }
                }
            }
            NP_Type::Result { ok, err } => {
                if internal_type_args.len() == 2 {
                    *ok = Box::new(internal_type_args[0]);
                    *err = Box::new(internal_type_args[1]);
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from(
                            "Result types require two arguments for contents: Result<Ok, Err>",
                        ),
                    });
                }
            }
            NP_Type::Option { some } => {
                if internal_type_args.len() == 1 {
                    *some = Box::new(internal_type_args[0]);
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from(
                            "Option types require one argument for contents: Option<X>",
                        ),
                    });
                }
            }
            NP_Type::Box { of, .. } => {
                if internal_type_args.len() == 1 {
                    *of = Box::new(internal_type_args[0]);
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Box require one argument for contents: Box<X>"),
                    });
                }
            }
            NP_Type::This { parent_schema_addr } => {
                *parent_schema_addr = parent_idx;
            }
            NP_Type::Array { of, len } => {
                if let Some(children) = contents_of_type {
                    let mut parse_idx: usize = 0;
                    *of = Box::new(parsed_schema.len());
                    if depth == 0 {
                        parse_idx = Self::parse_single_type(
                            source,
                            children,
                            parse_idx,
                            depth + 1,
                            this_schema_addr,
                            &result_schema.generics,
                            type_idx,
                            parsed_schema,
                        )?;
                    } else {
                        parse_idx = Self::parse_single_type(
                            source,
                            children,
                            parse_idx,
                            depth + 1,
                            parent_idx,
                            &generics,
                            type_idx,
                            parsed_schema,
                        )?;
                    }
                    parse_idx += 1;

                    if let AST::semicolon = &children[parse_idx] {
                        parse_idx += 1;
                    } else {
                        return Err(NP_Error::Custom {
                            message: String::from("Error parsing array type!"),
                        });
                    }

                    if let AST::number { addr } = &children[parse_idx] {
                        if let Ok(length) = addr.read(source).parse::<u16>() {
                            *len = length;
                        } else {
                            return Err(NP_Error::Custom {
                                message: String::from("Error parsing array type!"),
                            });
                        }
                    } else {
                        return Err(NP_Error::Custom {
                            message: String::from("Error parsing array type!"),
                        });
                    }
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Missing array items!"),
                    });
                }
            }
            NP_Type::Tuple { children } => {
                // handle this condition
                // tuple ( /* .. */ )
                if is_implicit == false {
                    let (next_index, parsed_children) =
                        Self::maybe_parse_children(ast, use_index, max_index, true);
                    use_index = next_index;
                    contents_of_type = parsed_children;
                }

                if let Some(children_ast) = contents_of_type {
                    let mut parse_idx: usize = 0;
                    // let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Value;

                    while parse_idx < children_ast.len() {
                        match parse_state {
                            ChildItemParseState::Key => { /* no keys here */ }
                            ChildItemParseState::Colon => { /* no colons here */ }
                            ChildItemParseState::Value => {
                                let schema_loc = parsed_schema.len();
                                children.push(schema_loc);
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        this_schema_addr,
                                        &result_schema.generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                } else {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        parent_idx,
                                        &generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            }
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            }
                                            AST::newline => {
                                                parse_idx += 1;
                                            }
                                            _ => {
                                                parse_state = ChildItemParseState::Value;
                                            }
                                        }
                                    } else {
                                        parse_state = ChildItemParseState::Finished
                                    }

                                    loop_max += 1; // prevent infinite loop
                                    if loop_max == u8::MAX {
                                        parse_state = ChildItemParseState::Finished
                                    }
                                }
                            }
                            ChildItemParseState::Finished => {
                                // nothing to do here
                            }
                        }
                    }
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Missing tuple children declaration!"),
                    });
                }
            }
            NP_Type::Impl { methods } => {
                let (next_index, children_ast_items) =
                    Self::maybe_parse_children(ast, use_index, max_index, false);
                use_index = next_index;
                contents_of_type = children_ast_items;

                if let Some(children_ast) = contents_of_type {
                    let mut parse_idx: usize = 0;
                    let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Key;

                    while parse_idx < children_ast.len() {
                        match parse_state {
                            ChildItemParseState::Key => {
                                if let AST::token { addr } = &children_ast[parse_idx] {
                                    key_ast = addr.clone();
                                    parse_state = ChildItemParseState::Value;
                                    parse_idx += 1;
                                } else {
                                    return Err(NP_Error::Custom {
                                        message: String::from("Error parsing impl child items!"),
                                    });
                                }
                            }
                            ChildItemParseState::Colon => { /* no colons here */ }
                            ChildItemParseState::Value => {
                                methods.set(key_ast.read(source), parsed_schema.len());
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        this_schema_addr,
                                        &result_schema.generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                } else {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        parent_idx,
                                        &generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            }
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            }
                                            AST::newline => {
                                                parse_idx += 1;
                                            }
                                            _ => {
                                                parse_state = ChildItemParseState::Key;
                                            }
                                        }
                                    } else {
                                        parse_state = ChildItemParseState::Finished
                                    }

                                    loop_max += 1; // prevent infinite loop
                                    if loop_max == u8::MAX {
                                        parse_state = ChildItemParseState::Finished
                                    }
                                }
                            }
                            ChildItemParseState::Finished => {
                                // nothing to do here
                            }
                        }
                    }
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Missing impl children declaration!"),
                    });
                }
            }
            NP_Type::Method { args, returns, .. } => {
                // parse args
                if let Some(children_ast) = contents_of_type {
                    let mut parse_idx: usize = 0;
                    let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Key;

                    while parse_idx < children_ast.len() {
                        match parse_state {
                            ChildItemParseState::Key => {
                                if let AST::token { addr } = &children_ast[parse_idx] {
                                    key_ast = addr.clone();
                                    parse_state = ChildItemParseState::Colon;
                                    parse_idx += 1;
                                } else {
                                    return Err(NP_Error::Custom {
                                        message: String::from("Error parsing method args!"),
                                    });
                                }
                            }
                            ChildItemParseState::Colon => {
                                match &children_ast[parse_idx] {
                                    AST::colon => {
                                        // named param
                                        parse_state = ChildItemParseState::Value;
                                        parse_idx += 1;
                                    }
                                    AST::comma => {
                                        // anonymous param (can only be self)
                                        let schema_loc = parsed_schema.len();
                                        if key_ast.read(source) == "self" {
                                            args.set("self", schema_loc);
                                            // if depth == 0 {
                                            //     parse_idx = Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, schema_len, &result_schema.generics, type_idx, parsed_schema)?;
                                            // } else {
                                            //     parse_idx = Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                            // }
                                            parse_state = ChildItemParseState::Comma;
                                            parse_idx += 1;
                                        } else {
                                            return Err(NP_Error::Custom {
                                                message: String::from(
                                                    "Error parsing method impl arguments!",
                                                ),
                                            });
                                        }
                                    }
                                    _ => {
                                        return Err(NP_Error::Custom {
                                            message: String::from(
                                                "Error parsing method impl arguments!",
                                            ),
                                        });
                                    }
                                }
                            }
                            ChildItemParseState::Value => {
                                let schema_loc = parsed_schema.len();
                                args.set(key_ast.read(source), schema_loc);
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        this_schema_addr,
                                        &result_schema.generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                } else {
                                    parse_idx = Self::parse_single_type(
                                        source,
                                        children_ast,
                                        parse_idx,
                                        depth + 1,
                                        parent_idx,
                                        &generics,
                                        type_idx,
                                        parsed_schema,
                                    )?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            }
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            }
                                            AST::newline => {
                                                parse_idx += 1;
                                            }
                                            _ => {
                                                parse_state = ChildItemParseState::Key;
                                            }
                                        }
                                    } else {
                                        parse_state = ChildItemParseState::Finished
                                    }

                                    loop_max += 1; // prevent infinite loop
                                    if loop_max == u8::MAX {
                                        parse_state = ChildItemParseState::Finished
                                    }
                                }
                            }
                            ChildItemParseState::Finished => {
                                // nothing to do here
                            }
                        }
                    }

                    // // last item in args was anonymous arg
                    // if let ChildItemParseState::Colon = parse_state {
                    //     let schema_loc = parsed_schema.len();
                    //     args.set("self", schema_loc);
                    //     if depth == 0 {
                    //         Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, schema_len, &result_schema.generics, type_idx, parsed_schema)?;
                    //     } else {
                    //         Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                    //     }
                    //     // *args_size += parsed_schema[schema_loc].kind.type_size();
                    // }
                }

                use_index += 1;

                // parse return value
                if let AST::arrow = &ast[use_index] {
                    use_index += 1;
                } else {
                    return Err(NP_Error::Custom {
                        message: String::from("Missing arrow from method declaration!"),
                    });
                }

                *returns = Box::new(parsed_schema.len());

                if depth == 0 {
                    use_index = Self::parse_single_type(
                        source,
                        ast,
                        use_index,
                        depth + 1,
                        this_schema_addr,
                        &result_schema.generics,
                        type_idx,
                        parsed_schema,
                    )?;
                } else {
                    use_index = Self::parse_single_type(
                        source,
                        ast,
                        use_index,
                        depth + 1,
                        parent_idx,
                        &generics,
                        type_idx,
                        parsed_schema,
                    )?;
                }

                use_index += 1;
            }
            NP_Type::Custom { generic_args, .. } => {
                if internal_type_args.len() > 0 {
                    *generic_args = Some(internal_type_args);
                }
            }
            NP_Type::Generic { .. } => { /* nothing to do */ }
            NP_Type::Simple_Enum { .. } => { /* unreachable */ }
            NP_Type::Unknown { .. } => { /* unreachable */ }
            NP_Type::RPC_Call { .. } => { /* unreachable */ }
            NP_Type::RPC_Return { .. } => { /* unreachable */ }
        }

        Self::maybe_error_on_generics(&result_schema)?;

        if depth == 0 {
            // set ID
            if let NP_Schema_Args::MAP(data) = &result_schema.args {
                if let Some(id) = data.get("id") {
                    if let NP_Schema_Args::NUMBER(data) = id {
                        if let Ok(id_num) = data.read(source).parse::<usize>() {
                            result_schema.id = Some(id_num);
                        }
                    }
                }
            }

            if result_schema.kind.val != NP_Type::Info {
                if None == result_schema.id {
                    if let NP_Type::Impl { .. } = &result_schema.kind.val {
                    } else {
                        return Err(NP_Error::Custom {
                            message: String::from("All top level types must have an id property!"),
                        });
                    }
                }
                if None == result_schema.name {
                    return Err(NP_Error::Custom {
                        message: String::from("All top level types must have a name!"),
                    });
                }
            } else {
                type_idx.set(
                    "__info",
                    NP_Schema_Index {
                        data: this_schema_addr,
                        methods: None,
                    },
                );
            }
        }

        if result_schema.kind.val == NP_Type::None {
            return Err(NP_Error::Custom {
                message: String::from("No valid type found!"),
            });
        }

        let is_simple_enum: Option<usize> =
            if let NP_Type::Enum { children, default } = &result_schema.kind.val {
                let mut is_simple = true;
                for (_, value) in children.iter() {
                    if let Some(_) = value {
                        is_simple = false;
                    }
                }
                if is_simple {
                    Some(*default)
                } else {
                    None
                }
            } else {
                None
            };
        if let Some(default) = is_simple_enum {
            result_schema.kind = NP_Schem_Kind::new(NP_Type::Simple_Enum {
                children: enum_keys,
                default,
            });
        }

        // set result schema
        parsed_schema[this_schema_addr] = result_schema;

        Ok(use_index)
    }

    pub fn read_ast_str(&self, ast_str: AST_STR) -> &str {
        ast_str.read_bytes(&self.source.as_slice())
    }

    fn bytes_to_args(
        buffer_loc: usize,
        buffer: &[u8],
    ) -> Result<(usize, NP_Schema_Args), NP_Error> {
        let mut index = buffer_loc;

        match buffer[index] {
            0 => Ok((index + 1, NP_Schema_Args::NULL)),
            1 => Ok((index + 1, NP_Schema_Args::TRUE)),
            2 => Ok((index + 1, NP_Schema_Args::FALSE)),
            3 => {
                // string
                let (new_index, ast_str) = AST_STR::from_bytes(index + 1, buffer)?;
                Ok((new_index + 1, NP_Schema_Args::STRING(ast_str)))
            }
            4 => {
                // number
                let (new_index, ast_str) = AST_STR::from_bytes(index + 1, buffer)?;
                Ok((new_index + 1, NP_Schema_Args::NUMBER(ast_str)))
            }
            5 => {
                // map
                let mut result: NP_OrderedMap<NP_Schema_Args> = NP_OrderedMap::new();
                index += 1;
                let mut item_length = buffer[index];
                index += 1;
                while item_length > 0 {
                    let (new_index, ast_str) = AST_STR::from_bytes(index + 1, buffer)?;
                    let (next_index, child_object) = Self::bytes_to_args(new_index, buffer)?;
                    result.set(ast_str.read_bytes(buffer), child_object);
                    index = next_index;
                    item_length -= 1;
                }

                Ok((index + 1, NP_Schema_Args::MAP(result)))
            }
            6 => {
                // list
                let mut result: Vec<NP_Schema_Args> = Vec::new();
                index += 1;
                let mut item_length = buffer[index];
                index += 1;
                while item_length > 0 {
                    let (next_index, child_object) = Self::bytes_to_args(index, buffer)?;
                    result.push(child_object);
                    index = next_index;
                    item_length -= 1;
                }
                Ok((index + 1, NP_Schema_Args::LIST(result)))
            }
            _ => Ok((index + 1, NP_Schema_Args::NULL)),
        }
    }

    fn args_to_bytes(
        &self,
        string_index: &mut NP_OrderedMap<AST_STR>,
        string_buffer: &mut Vec<u8>,
        args: &NP_Schema_Args,
    ) -> Result<Vec<u8>, NP_Error> {
        let mut result = Vec::new();

        match args {
            NP_Schema_Args::NULL => {
                result.extend_from_slice(&[0u8]);
            }
            NP_Schema_Args::TRUE => {
                result.extend_from_slice(&[1u8]);
            }
            NP_Schema_Args::FALSE => {
                result.extend_from_slice(&[2u8]);
            }
            NP_Schema_Args::STRING(ast_str) => {
                result.extend_from_slice(&[3u8]);
                let string_value = ast_str.read_bytes(&self.source);
                if let Some(target_ast) = string_index.get(string_value) {
                    result.extend_from_slice(&target_ast.to_bytes());
                } else {
                    let new_ast = AST_STR {
                        start: string_buffer.len(),
                        end: string_buffer.len() + string_value.len(),
                    };
                    string_buffer.extend_from_slice(string_value.as_bytes());
                    result.extend_from_slice(&new_ast.to_bytes());
                    string_index.set(string_value, new_ast);
                }
            }
            NP_Schema_Args::NUMBER(ast_str) => {
                result.extend_from_slice(&[4u8]);
                let string_value = ast_str.read_bytes(&self.source);
                if let Some(target_ast) = string_index.get(string_value) {
                    result.extend_from_slice(&target_ast.to_bytes());
                } else {
                    let new_ast = AST_STR {
                        start: string_buffer.len(),
                        end: string_buffer.len() + string_value.len(),
                    };
                    string_buffer.extend_from_slice(string_value.as_bytes());
                    result.extend_from_slice(&new_ast.to_bytes());
                    string_index.set(string_value, new_ast);
                }
            }
            NP_Schema_Args::MAP(map) => {
                result.extend_from_slice(&[5u8]);
                result.extend_from_slice(&[map.data.len() as u8]);

                for (key, value) in map.iter() {
                    // set key
                    if let Some(target_ast) = string_index.get(key) {
                        result.extend_from_slice(&target_ast.to_bytes());
                    } else {
                        let new_ast = AST_STR {
                            start: string_buffer.len(),
                            end: string_buffer.len() + key.len(),
                        };
                        result.extend_from_slice(&new_ast.to_bytes());
                        string_buffer.extend_from_slice(key.as_bytes());
                        string_index.set(key, new_ast);
                    }
                    let value_bytes = self.args_to_bytes(string_index, string_buffer, value)?;
                    result.extend_from_slice(&value_bytes);
                }
            }
            NP_Schema_Args::LIST(list) => {
                result.extend_from_slice(&[6u8]);
                result.extend_from_slice(&[list.len() as u8]);

                for value in list.iter() {
                    let value_bytes = self.args_to_bytes(string_index, string_buffer, value)?;
                    result.extend_from_slice(&value_bytes);
                }
            }
        }

        return Ok(result);
    }

    // pub fn from_bytes(bytes: &[u8]) -> Result<Self, NP_Error> {
    //     let mut result: NP_Schema = Default::default();

    //     result.source = Vec::from(bytes);

    //     let ptr = &bytes[0];
    //     let mut parse_pointer: usize = le_bytes_read!(u16, ptr) as usize;

    //     let ptr = &bytes[parse_pointer];
    //     result.unique_id = le_bytes_read!(u32, ptr);
    //     parse_pointer += 4;

    //     let ptr = &bytes[parse_pointer];
    //     let mut schema_len = le_bytes_read!(u16, ptr);
    //     parse_pointer += 2;

    //     while schema_len > 0 {
    //         let mut new_schema: NP_Type = Default::default();

    //         if bytes[parse_pointer] > 60 {
    //             // generics only
    //             let type_idx = bytes[parse_pointer] - 60;
    //             parse_pointer += 1;
    //             new_schema.kind = NP_Type::from(type_idx);

    //             // parse generics
    //             if bytes[parse_pointer] > 150 {
    //                 let args_length = (bytes[parse_pointer] - 150) as usize;
    //                 let ast_args = vec![AST_STR { start: 0, end: 0 }; args_length];
    //                 new_schema.generics = NP_Parsed_Generics::Arguments(0, ast_args);

    //                 parse_pointer += 1;
    //             } else if bytes[parse_pointer] > 0 {
    //                 let mut types_length = (bytes[parse_pointer] - 1) as usize;
    //                 parse_pointer += 1;
    //                 let mut types_vec: Vec<usize> = Vec::new();
    //                 while types_length > 0 {
    //                     let ptr = &bytes[parse_pointer];
    //                     types_vec.push(le_bytes_read!(u16, ptr) as usize);
    //                     parse_pointer += 2;
    //                     types_length -= 1;
    //                 }
    //                 new_schema.generics = NP_Parsed_Generics::Types(types_vec);
    //             }
    //         } else if bytes[parse_pointer] > 1 {
    //             // simple type
    //             let type_idx = bytes[parse_pointer] - 1;
    //             parse_pointer += 1;
    //             new_schema.kind = NP_Type::from(type_idx);
    //         } else {
    //             // slower path for more complicated types
    //             parse_pointer += 1;
    //             let type_idx = bytes[parse_pointer];
    //             parse_pointer += 1;
    //             new_schema.kind = NP_Type::from(type_idx);

    //             if bytes[parse_pointer] > 150 {
    //                 // generics
    //                 let args_length = (bytes[parse_pointer] - 150) as usize;
    //                 let ast_args = vec![AST_STR { start: 0, end: 0 }; args_length];
    //                 new_schema.generics = NP_Parsed_Generics::Arguments(0, ast_args);

    //                 parse_pointer += 1;
    //             } else if bytes[parse_pointer] > 0 {
    //                 let mut types_length = (bytes[parse_pointer] - 1) as usize;
    //                 parse_pointer += 1;
    //                 let mut types_vec: Vec<usize> = Vec::new();
    //                 while types_length > 0 {
    //                     let ptr = &bytes[parse_pointer];
    //                     types_vec.push(le_bytes_read!(u16, ptr) as usize);
    //                     parse_pointer += 2;
    //                     types_length -= 1;
    //                 }
    //                 new_schema.generics = NP_Parsed_Generics::Types(types_vec);
    //             }

    //             if bytes[parse_pointer] == 0 {
    //                 // name
    //                 parse_pointer += 1;
    //             } else {
    //                 // name found
    //                 parse_pointer += 1;

    //                 let (next_index, name_ast) = AST_STR::from_bytes(parse_pointer, bytes)?;
    //                 parse_pointer = next_index;
    //                 new_schema.name = Some(name_ast);

    //                 parse_pointer += 1;
    //             }

    //             if bytes[parse_pointer] == 0 {
    //                 // id
    //                 parse_pointer += 1;
    //             } else {
    //                 parse_pointer += 1;
    //                 let ptr = &bytes[parse_pointer];
    //                 let item_id = le_bytes_read!(u16, ptr);
    //                 new_schema.id = Some(item_id as usize);
    //                 parse_pointer += 2;
    //             }

    //             if bytes[parse_pointer] == 0 {
    //                 // args
    //                 parse_pointer += 1;
    //             } else {
    //                 parse_pointer += 1;
    //                 let (new_index, args) = Self::bytes_to_args(parse_pointer, bytes)?;
    //                 new_schema.arguments = args;
    //                 parse_pointer = new_index;
    //             }

    //             match &mut new_schema.kind {
    //                 NP_Type::None => {}
    //                 NP_Type::Any { .. } => {}
    //                 NP_Type::Info => {}
    //                 NP_Type::String {
    //                     size,
    //                     default,
    //                     casing,
    //                     max_len,
    //                 } => {
    //                     if default.start == 0 && default.end == 0 {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[1u8]);
    //                         let default_string = default.read_bytes(&self.source);
    //                         if let Some(index_pos) = string_index.get(default_string) {
    //                             schema_section.extend_from_slice(&index_pos.to_bytes());
    //                         } else {
    //                             let new_string_ast = AST_STR {
    //                                 start: result.len(),
    //                                 end: result.len() + default_string.len(),
    //                             };
    //                             result.extend_from_slice(default_string.as_bytes());
    //                             string_index.set(default_string, new_string_ast)?;
    //                             schema_section.extend_from_slice(&new_string_ast.to_bytes());
    //                         }
    //                     }

    //                     match casing {
    //                         NP_String_Casing::None => {
    //                             schema_section.extend_from_slice(&[0u8]);
    //                         }
    //                         NP_String_Casing::Uppercase => {
    //                             schema_section.extend_from_slice(&[1u8]);
    //                         }
    //                         NP_String_Casing::Lowercase => {
    //                             schema_section.extend_from_slice(&[2u8]);
    //                         }
    //                     }

    //                     if let Some(len) = max_len {
    //                         schema_section.extend_from_slice(&((len + 1) as u16).to_le_bytes());
    //                     } else {
    //                         schema_section.extend_from_slice(&[0u8, 0u8]);
    //                     }
    //                 }
    //                 NP_Type::Char { size, default } => {
    //                     if default == &(0 as char) {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[*default as u8 + 1]);
    //                     }
    //                 }
    //                 NP_Type::Int8 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(i8, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Int16 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(i16, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Int32 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(i32, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Int64 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(i64, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Uint8 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(u8, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Uint16 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(u16, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Uint32 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(u32, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Uint64 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(u64, default, min, max, schema_section);
    //                 }
    //                 NP_Type::f32 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(f32, default, min, max, schema_section);
    //                 }
    //                 NP_Type::f64 {
    //                     size,
    //                     default,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_number!(f64, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Dec32 {
    //                     size,
    //                     default,
    //                     exp,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_dec!(exp, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Dec64 {
    //                     size,
    //                     default,
    //                     exp,
    //                     min,
    //                     max,
    //                 } => {
    //                     schema_bytes_dec!(exp, default, min, max, schema_section);
    //                 }
    //                 NP_Type::Boolean { size, default } => {
    //                     if *default == false {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[1u8]);
    //                     }
    //                 }
    //                 NP_Type::Geo32 { size, default } => {
    //                     if default.0 == 0 && default.1 == 0 {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[1u8]);
    //                         schema_section.extend_from_slice(&default.0.to_le_bytes());
    //                         schema_section.extend_from_slice(&default.1.to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Geo64 { size, default } => {
    //                     if default.0 == 0 && default.1 == 0 {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[1u8]);
    //                         schema_section.extend_from_slice(&default.0.to_le_bytes());
    //                         schema_section.extend_from_slice(&default.1.to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Geo128 { size, default } => {
    //                     if default.0 == 0 && default.1 == 0 {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[1u8]);
    //                         schema_section.extend_from_slice(&default.0.to_le_bytes());
    //                         schema_section.extend_from_slice(&default.1.to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Uuid { .. } => {}
    //                 NP_Type::Ulid { .. } => {}
    //                 NP_Type::Date { .. } => {}
    //                 NP_Type::Enum {
    //                     size,
    //                     children,
    //                     default,
    //                 } => {
    //                     schema_section.extend_from_slice(&(*size as u16).to_le_bytes());
    //                     schema_section.extend_from_slice(&[children.keys().len() as u8]);

    //                     for (key, value) in children.iter() {
    //                         if let Some(target_ast) = string_index.get(key) {
    //                             schema_section.extend_from_slice(&target_ast.to_bytes());
    //                         } else {
    //                             let new_ast = AST_STR {
    //                                 start: result.len(),
    //                                 end: result.len() + key.len(),
    //                             };
    //                             schema_section.extend_from_slice(&new_ast.to_bytes());
    //                             string_index.set(key, new_ast)?;
    //                             result.extend_from_slice(key.as_bytes());
    //                         }

    //                         if let Some(opt) = value {
    //                             schema_section
    //                                 .extend_from_slice(&((*opt as u16) + 1).to_le_bytes());
    //                         } else {
    //                             schema_section.extend_from_slice(&(0u16).to_le_bytes());
    //                         }
    //                     }

    //                     if let Some(def) = default {
    //                         schema_section.extend_from_slice(&[*def as u8 + 1]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     }
    //                 }
    //                 NP_Type::Struct { size, children } => {
    //                     schema_section.extend_from_slice(&(*size as u16).to_le_bytes());
    //                     schema_section.extend_from_slice(&[children.keys().len() as u8]);

    //                     for (key, value) in children.iter() {
    //                         if let Some(target_ast) = string_index.get(key) {
    //                             schema_section.extend_from_slice(&target_ast.to_bytes());
    //                         } else {
    //                             let new_ast = AST_STR {
    //                                 start: result.len(),
    //                                 end: result.len() + key.len(),
    //                             };
    //                             schema_section.extend_from_slice(&new_ast.to_bytes());
    //                             string_index.set(key, new_ast)?;
    //                             result.extend_from_slice(key.as_bytes());
    //                         }
    //                         schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Map { .. } => {}
    //                 NP_Type::Vec { .. } => {}
    //                 NP_Type::Result { .. } => {}
    //                 NP_Type::Option { .. } => {}
    //                 NP_Type::Array { .. } => {}
    //                 NP_Type::Tuple { size, children } => {
    //                     schema_section.extend_from_slice(&(*size as u16).to_le_bytes());
    //                     schema_section.extend_from_slice(&[children.len() as u8]);

    //                     for value in children.iter() {
    //                         schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Impl { children } => {
    //                     schema_section.extend_from_slice(&[children.keys().len() as u8]);

    //                     for (key, value) in children.iter() {
    //                         if let Some(target_ast) = string_index.get(key) {
    //                             schema_section.extend_from_slice(&target_ast.to_bytes());
    //                         } else {
    //                             let new_ast = AST_STR {
    //                                 start: result.len(),
    //                                 end: result.len() + key.len(),
    //                             };
    //                             schema_section.extend_from_slice(&new_ast.to_bytes());
    //                             string_index.set(key, new_ast)?;
    //                             result.extend_from_slice(key.as_bytes());
    //                         }
    //                         schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Fn_Self { idx } => {
    //                     schema_section.extend_from_slice(&(*idx as u16).to_le_bytes());
    //                 }
    //                 NP_Type::Method { args, returns } => {
    //                     schema_section.extend_from_slice(&(*returns as u16).to_le_bytes());
    //                     schema_section.extend_from_slice(&[args.keys().len() as u8]);

    //                     for (key, value) in args.iter() {
    //                         if let Some(target_ast) = string_index.get(key) {
    //                             schema_section.extend_from_slice(&target_ast.to_bytes());
    //                         } else {
    //                             let new_ast = AST_STR {
    //                                 start: result.len(),
    //                                 end: result.len() + key.len(),
    //                             };
    //                             schema_section.extend_from_slice(&new_ast.to_bytes());
    //                             string_index.set(key, new_ast)?;
    //                             result.extend_from_slice(key.as_bytes());
    //                         }
    //                         schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
    //                     }
    //                 }
    //                 NP_Type::Generic {
    //                     size,
    //                     parent_scham_addr,
    //                     generic_idx,
    //                 } => {
    //                     schema_section
    //                         .extend_from_slice(&(*parent_scham_addr as u16).to_le_bytes());
    //                     schema_section.extend_from_slice(&(*generic_idx as u16).to_le_bytes());
    //                 }
    //                 NP_Type::Custom { size, type_idx } => {
    //                     schema_section.extend_from_slice(&(*type_idx as u16).to_le_bytes());
    //                 }
    //                 NP_Type::Box { .. } => {}
    //                 NP_Type::Simple_Enum {
    //                     size,
    //                     children,
    //                     default,
    //                 } => {
    //                     schema_section.extend_from_slice(&[children.len() as u8]);

    //                     for value in children.iter() {
    //                         if let Some(target_ast) = string_index.get(value) {
    //                             schema_section.extend_from_slice(&target_ast.to_bytes());
    //                         } else {
    //                             let new_ast = AST_STR {
    //                                 start: result.len(),
    //                                 end: result.len() + value.len(),
    //                             };
    //                             result.extend_from_slice(value.as_bytes());
    //                             schema_section.extend_from_slice(&new_ast.to_bytes());
    //                             string_index.set(value, new_ast)?;
    //                         }
    //                     }

    //                     if let Some(def) = default {
    //                         schema_section.extend_from_slice(&[*def as u8 + 1]);
    //                     } else {
    //                         schema_section.extend_from_slice(&[0u8]);
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }

    //         result.schemas.push(new_schema);
    //         schema_len -= 1;
    //     }

    //     Ok(result)
    // }

    // compile schema into bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, NP_Error> {
        let mut result: Vec<u8> = Vec::new();

        result.extend_from_slice(&0u16.to_le_bytes());

        let mut string_index: NP_OrderedMap<AST_STR> = NP_OrderedMap::new();

        let mut schema_section: Vec<u8> = Vec::new();

        for schema in &self.schemas {
            let schema_data = schema.kind.type_info();

            let is_complex_type = schema_data.0 == 24
                || schema_data.0 == 25
                || schema_data.0 == 31
                || schema_data.0 == 33
                || schema_data.0 == 34
                || schema_data.0 == 35
                || schema_data.0 == 38;
            let has_no_data_points = schema.name == None
                && schema.id == None
                && schema.arguments == NP_Schema_Args::NULL;
            let has_no_generics = schema.generics == NP_Parsed_Generics::None;

            if is_complex_type == false && has_no_data_points == true && has_no_generics == true {
                // no generics, simple type, no arguments
                schema_section.extend_from_slice(&[(schema_data.0 + 1) as u8]);
            } else if is_complex_type == false && has_no_data_points == true {
                // type just has generics

                schema_section.extend_from_slice(&[(schema_data.0 + 60) as u8]);

                match &schema.generics {
                    NP_Parsed_Generics::None => {
                        schema_section.extend_from_slice(&[0u8]);
                    }
                    NP_Parsed_Generics::Types(types) => {
                        schema_section.extend_from_slice(&[types.len() as u8 + 1]);
                        for type_idx in types.iter() {
                            schema_section.extend_from_slice(&(*type_idx as u16).to_le_bytes());
                        }
                    }
                    NP_Parsed_Generics::Arguments(parent, args) => {
                        schema_section.extend_from_slice(&[args.len() as u8 + 150]);
                    }
                }
            } else {
                schema_section.extend_from_slice(&[0u8]); // complex parse path marker

                // type info
                schema_section.extend_from_slice(&[schema_data.0 as u8]);

                // generics
                match &schema.generics {
                    NP_Parsed_Generics::None => {
                        schema_section.extend_from_slice(&[0u8]);
                    }
                    NP_Parsed_Generics::Types(types) => {
                        schema_section.extend_from_slice(&[types.len() as u8 + 1]);
                        for type_idx in types.iter() {
                            schema_section.extend_from_slice(&(*type_idx as u16).to_le_bytes());
                        }
                    }
                    NP_Parsed_Generics::Arguments(parent, args) => {
                        schema_section.extend_from_slice(&[args.len() as u8 + 150]);
                    }
                }

                // schema name
                if let Some(source_pos) = schema.name {
                    schema_section.extend_from_slice(&[1u8]);

                    let schema_name = source_pos.read_bytes(&self.source);

                    if let Some(index_pos) = string_index.get(schema_name) {
                        schema_section.extend_from_slice(&index_pos.to_bytes());
                    } else {
                        let new_string_ast = AST_STR {
                            start: result.len(),
                            end: result.len() + schema_name.len(),
                        };
                        result.extend_from_slice(schema_name.as_bytes());
                        string_index.set(schema_name, new_string_ast)?;
                        schema_section.extend_from_slice(&new_string_ast.to_bytes());
                    }
                } else {
                    schema_section.extend_from_slice(&[0u8]);
                }

                // schema id
                if let Some(id) = schema.id {
                    schema_section.extend_from_slice(&[1u8]);
                    schema_section.extend_from_slice(&(id as u16).to_le_bytes());
                } else {
                    schema_section.extend_from_slice(&[0u8]);
                }

                // schema args
                if let NP_Schema_Args::NULL = schema.arguments {
                    schema_section.extend_from_slice(&[0u8]);
                } else {
                    schema_section.extend_from_slice(&[1u8]);
                    schema_section.extend_from_slice(&self.args_to_bytes(
                        &mut string_index,
                        &mut result,
                        &schema.arguments,
                    )?);
                }

                // // schema offset
                // schema_section.extend_from_slice(&(schema.offset as u16).to_le_bytes());

                match &schema.kind {
                    NP_Type::None => {}
                    NP_Type::Any { .. } => {}
                    NP_Type::Info => {}
                    NP_Type::String {
                        default,
                        casing,
                        max_len,
                    } => {
                        if default.start == 0 && default.end == 0 {
                            schema_section.extend_from_slice(&[0u8]);
                        } else {
                            schema_section.extend_from_slice(&[1u8]);
                            let default_string = default.read_bytes(&self.source);
                            if let Some(index_pos) = string_index.get(default_string) {
                                schema_section.extend_from_slice(&index_pos.to_bytes());
                            } else {
                                let new_string_ast = AST_STR {
                                    start: result.len(),
                                    end: result.len() + default_string.len(),
                                };
                                result.extend_from_slice(default_string.as_bytes());
                                string_index.set(default_string, new_string_ast)?;
                                schema_section.extend_from_slice(&new_string_ast.to_bytes());
                            }
                        }

                        match casing {
                            NP_String_Casing::None => {
                                schema_section.extend_from_slice(&[0u8]);
                            }
                            NP_String_Casing::Uppercase => {
                                schema_section.extend_from_slice(&[1u8]);
                            }
                            NP_String_Casing::Lowercase => {
                                schema_section.extend_from_slice(&[2u8]);
                            }
                        }

                        if let Some(len) = max_len {
                            schema_section.extend_from_slice(&((len + 1) as u16).to_le_bytes());
                        } else {
                            schema_section.extend_from_slice(&[0u8, 0u8]);
                        }
                    }
                    NP_Type::Char { default } => {
                        if default == &(0 as char) {
                            schema_section.extend_from_slice(&[0u8]);
                        } else {
                            schema_section.extend_from_slice(&[*default as u8 + 1]);
                        }
                    }
                    NP_Type::Int8 { default, min, max } => {
                        schema_bytes_number!(i8, default, min, max, schema_section);
                    }
                    NP_Type::Int16 { default, min, max } => {
                        schema_bytes_number!(i16, default, min, max, schema_section);
                    }
                    NP_Type::Int32 { default, min, max } => {
                        schema_bytes_number!(i32, default, min, max, schema_section);
                    }
                    NP_Type::Int64 { default, min, max } => {
                        schema_bytes_number!(i64, default, min, max, schema_section);
                    }
                    NP_Type::Uint8 { default, min, max } => {
                        schema_bytes_number!(u8, default, min, max, schema_section);
                    }
                    NP_Type::Uint16 { default, min, max } => {
                        schema_bytes_number!(u16, default, min, max, schema_section);
                    }
                    NP_Type::Uint32 { default, min, max } => {
                        schema_bytes_number!(u32, default, min, max, schema_section);
                    }
                    NP_Type::Uint64 { default, min, max } => {
                        schema_bytes_number!(u64, default, min, max, schema_section);
                    }
                    NP_Type::f32 { default, min, max } => {
                        schema_bytes_number!(f32, default, min, max, schema_section);
                    }
                    NP_Type::f64 { default, min, max } => {
                        schema_bytes_number!(f64, default, min, max, schema_section);
                    }
                    NP_Type::Dec32 {
                        default,
                        exp,
                        min,
                        max,
                    } => {
                        schema_bytes_dec!(exp, default, min, max, schema_section);
                    }
                    NP_Type::Dec64 {
                        default,
                        exp,
                        min,
                        max,
                    } => {
                        schema_bytes_dec!(exp, default, min, max, schema_section);
                    }
                    NP_Type::Boolean { default } => {
                        if *default == false {
                            schema_section.extend_from_slice(&[0u8]);
                        } else {
                            schema_section.extend_from_slice(&[1u8]);
                        }
                    }
                    NP_Type::Geo32 { default } => {
                        if default.0 == 0 && default.1 == 0 {
                            schema_section.extend_from_slice(&[0u8]);
                        } else {
                            schema_section.extend_from_slice(&[1u8]);
                            schema_section.extend_from_slice(&default.0.to_le_bytes());
                            schema_section.extend_from_slice(&default.1.to_le_bytes());
                        }
                    }
                    NP_Type::Geo64 { default } => {
                        if default.0 == 0 && default.1 == 0 {
                            schema_section.extend_from_slice(&[0u8]);
                        } else {
                            schema_section.extend_from_slice(&[1u8]);
                            schema_section.extend_from_slice(&default.0.to_le_bytes());
                            schema_section.extend_from_slice(&default.1.to_le_bytes());
                        }
                    }
                    NP_Type::Geo128 { default } => {
                        if default.0 == 0 && default.1 == 0 {
                            schema_section.extend_from_slice(&[0u8]);
                        } else {
                            schema_section.extend_from_slice(&[1u8]);
                            schema_section.extend_from_slice(&default.0.to_le_bytes());
                            schema_section.extend_from_slice(&default.1.to_le_bytes());
                        }
                    }
                    NP_Type::Uuid { .. } => {}
                    NP_Type::Ulid { .. } => {}
                    NP_Type::Date { .. } => {}
                    NP_Type::Enum { children, default } => {
                        schema_section.extend_from_slice(&[children.keys().len() as u8]);

                        for (key, value) in children.iter() {
                            if let Some(target_ast) = string_index.get(key) {
                                schema_section.extend_from_slice(&target_ast.to_bytes());
                            } else {
                                let new_ast = AST_STR {
                                    start: result.len(),
                                    end: result.len() + key.len(),
                                };
                                schema_section.extend_from_slice(&new_ast.to_bytes());
                                string_index.set(key, new_ast)?;
                                result.extend_from_slice(key.as_bytes());
                            }

                            if let Some(opt) = value {
                                schema_section
                                    .extend_from_slice(&((*opt as u16) + 1).to_le_bytes());
                            } else {
                                schema_section.extend_from_slice(&(0u16).to_le_bytes());
                            }
                        }

                        if let Some(def) = default {
                            schema_section.extend_from_slice(&[*def as u8 + 1]);
                        } else {
                            schema_section.extend_from_slice(&[0u8]);
                        }
                    }
                    NP_Type::Struct { children } => {
                        // schema_section.extend_from_slice(&(*size as u16).to_le_bytes());
                        schema_section.extend_from_slice(&[children.keys().len() as u8]);

                        for (key, value) in children.iter() {
                            if let Some(target_ast) = string_index.get(key) {
                                schema_section.extend_from_slice(&target_ast.to_bytes());
                            } else {
                                let new_ast = AST_STR {
                                    start: result.len(),
                                    end: result.len() + key.len(),
                                };
                                schema_section.extend_from_slice(&new_ast.to_bytes());
                                string_index.set(key, new_ast)?;
                                result.extend_from_slice(key.as_bytes());
                            }
                            schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
                        }
                    }
                    NP_Type::Map { .. } => {}
                    NP_Type::Vec { .. } => {}
                    NP_Type::Result { .. } => {}
                    NP_Type::Option { .. } => {}
                    NP_Type::Array { .. } => {}
                    NP_Type::Tuple { children } => {
                        // schema_section.extend_from_slice(&(*size as u16).to_le_bytes());
                        schema_section.extend_from_slice(&[children.len() as u8]);

                        for value in children.iter() {
                            schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
                        }
                    }
                    NP_Type::Impl { children } => {
                        schema_section.extend_from_slice(&[children.keys().len() as u8]);

                        for (key, value) in children.iter() {
                            if let Some(target_ast) = string_index.get(key) {
                                schema_section.extend_from_slice(&target_ast.to_bytes());
                            } else {
                                let new_ast = AST_STR {
                                    start: result.len(),
                                    end: result.len() + key.len(),
                                };
                                schema_section.extend_from_slice(&new_ast.to_bytes());
                                string_index.set(key, new_ast)?;
                                result.extend_from_slice(key.as_bytes());
                            }
                            schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
                        }
                    }
                    NP_Type::Fn_Self { idx } => {
                        schema_section.extend_from_slice(&(*idx as u16).to_le_bytes());
                    }
                    NP_Type::Method { args, returns } => {
                        schema_section.extend_from_slice(&(*returns as u16).to_le_bytes());
                        schema_section.extend_from_slice(&[args.keys().len() as u8]);

                        for (key, value) in args.iter() {
                            if let Some(target_ast) = string_index.get(key) {
                                schema_section.extend_from_slice(&target_ast.to_bytes());
                            } else {
                                let new_ast = AST_STR {
                                    start: result.len(),
                                    end: result.len() + key.len(),
                                };
                                schema_section.extend_from_slice(&new_ast.to_bytes());
                                string_index.set(key, new_ast)?;
                                result.extend_from_slice(key.as_bytes());
                            }
                            schema_section.extend_from_slice(&(*value as u16).to_le_bytes());
                        }
                    }
                    NP_Type::Generic {
                        parent_scham_addr,
                        generic_idx,
                    } => {
                        schema_section
                            .extend_from_slice(&(*parent_scham_addr as u16).to_le_bytes());
                        schema_section.extend_from_slice(&(*generic_idx as u16).to_le_bytes());
                    }
                    NP_Type::Custom { type_idx } => {
                        schema_section.extend_from_slice(&(*type_idx as u16).to_le_bytes());
                    }
                    NP_Type::Box { .. } => {}
                    NP_Type::Simple_Enum { children, default } => {
                        schema_section.extend_from_slice(&[children.len() as u8]);

                        for value in children.iter() {
                            if let Some(target_ast) = string_index.get(value) {
                                schema_section.extend_from_slice(&target_ast.to_bytes());
                            } else {
                                let new_ast = AST_STR {
                                    start: result.len(),
                                    end: result.len() + value.len(),
                                };
                                result.extend_from_slice(value.as_bytes());
                                schema_section.extend_from_slice(&new_ast.to_bytes());
                                string_index.set(value, new_ast)?;
                            }
                        }

                        if let Some(def) = default {
                            schema_section.extend_from_slice(&[*def as u8 + 1]);
                        } else {
                            schema_section.extend_from_slice(&[0u8]);
                        }
                    }
                }
            }
        }

        // write string section length into buffer
        let val = &(result.len() as u16);
        let ptr = &mut result[0];
        le_bytes_write!(u16, ptr, val);

        // write schema section into buffer
        result.extend_from_slice(&self.unique_id.to_le_bytes());
        result.extend_from_slice(&(self.schemas.len() as u16).to_le_bytes());
        result.extend_from_slice(&schema_section[..]);

        Ok(result)
    }
}
