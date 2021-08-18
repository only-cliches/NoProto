use crate::schema::{NP_Schema, NP_Schema_Index, NP_Parsed_Schema, NP_Schema_Type, POINTER_SIZE, NP_String_Casing};
use crate::error::NP_Error;
use crate::schema::ast_parser::{AST, AST_STR};
use alloc::prelude::v1::{String, Vec};
use crate::hashmap::NP_HashMap;
use crate::schema::schema_args::NP_Schema_Args;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum ChildItemParseState {
    Key,
    Colon,
    Value,
    Comma,
    Finished
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
    }
}

macro_rules! schema_geo {
    ($source: tt, $arguments: tt, $kind: ty, $default: tt, $deviser: tt) => {

        if let NP_Schema_Args::MAP (args_map) = &$arguments {
            if let Some(NP_Schema_Args::MAP (lat_lng )) = args_map.get("default") {
                if let Some(NP_Schema_Args::NUMBER ( lat )) = lat_lng.get("lat") {
                    if let Some(NP_Schema_Args::NUMBER ( lng )) = lat_lng.get("lng") {
                        if let Ok(lat_parsed) = lat.read($source).parse::<f64>()  {
                            if let Ok(lng_parsed) = lng.read($source).parse::<f64>()  {
                                *$default = ((lat_parsed * $deviser) as $kind, (lng_parsed * $deviser) as $kind);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
impl NP_Schema {
    pub fn parse<S>(input: S) -> Result<Self, NP_Error> where S: AsRef<str> {
        let ast = AST::parse(input.as_ref())?;

        let mut parse_idx: usize = 0;
        let mut parse_schema: Vec<NP_Parsed_Schema> = Vec::new();
        let mut type_idx: NP_HashMap<NP_Schema_Index> = NP_HashMap::new();

        let top_generics = None;

        let mut max_loop:u16 = 0;

        while parse_idx < ast.len() && max_loop < u16::MAX {
            max_loop += 1;

            if ast[parse_idx] == AST::newline || ast[parse_idx] == AST::semicolon {
                parse_idx += 1;
            } else {
                parse_idx = Self::parse_single_type(input.as_ref(), &ast, parse_idx, 0, 0, &top_generics, &mut type_idx, &mut parse_schema)?;
                parse_idx += 1;
            }
        }

        // build ID index
        let mut max_id: u16 = 0;
        for schema in &parse_schema {
            if let Some(id) = schema.id {
                max_id = u16::max(id, max_id);
            }
        }

        max_id += 1;

        let mut id_idx: Vec<NP_Schema_Index> = vec![NP_Schema_Index::default(); max_id as usize];

        for schema in &parse_schema {
            if let Some(id) = schema.id {
                if let Some(name) = schema.name {
                    if let Some(schema_index) = type_idx.get(name.read(input.as_ref())) {
                        id_idx[id as usize] = schema_index.clone();
                    }
                }
            }
        }

        Ok(Self {
            source: String::from(input.as_ref()).into_bytes(),
            schemas: parse_schema,
            name_index: type_idx,
            id_index: id_idx
        })
    }

    fn maybe_error_on_generics(result_schema: &NP_Parsed_Schema) -> Result<(), NP_Error> {
        if None != result_schema.self_generics {
            match &result_schema.data_type {
                NP_Schema_Type::Enum { .. } => {}
                NP_Schema_Type::Struct { .. } => {}
                NP_Schema_Type::Tuple { .. } => {}
                NP_Schema_Type::Impl { .. } => {},
                _ => {
                    let mut msg = String::from("Error: this type does not support generic arguments: ");
                    msg.push_str(result_schema.data_type.type_info().1);
                    return Err(NP_Error::Custom { message: msg})
                }
                // NP_Schema_Type::Generic { .. } => {}
                // NP_Schema_Type::Custom { .. } => {}
            }
        }

        Ok(())
    }

    fn maybe_parse_children(ast: &Vec<AST>, index: usize, max_index: usize, is_tuple: bool) -> (usize, Option<&Vec<AST>>) {
        if index + 1 >= max_index {
            return (index, None);
        }

        if is_tuple {
            match &ast[index + 1] {
                AST::parans { items } => {
                    (index + 1, Some(items))
                },
                _ => {
                    (index, None)
                }
            }
        } else {
            match &ast[index + 1] {
                AST::curly { items } => {
                    (index + 1, Some(items))
                },
                _ => {
                    (index, None)
                }
            }
        }
    }

    fn maybe_parse_title(ast: &Vec<AST>, index: usize, max_index: usize, result_schema: &mut NP_Parsed_Schema) -> usize {

        if index + 1 >= max_index {
            return index;
        }

        match &ast[index + 1] {
            AST::token { addr } => {
                result_schema.name = Some(addr.clone());
                index + 1
            },
            _ => {
                index
            }
        }
    }

    fn maybe_parse_generics(ast: &Vec<AST>, index: usize, max_index: usize, schema_len: usize, result_schema: &mut NP_Parsed_Schema) -> Result<usize, NP_Error> {
        if index + 1 >= max_index {
            return Ok(index);
        }

        match &ast[index + 1] {
            AST::xml { items } => {
                let mut generics: Vec<AST_STR> = Vec::new();

                for generic in items.iter() {
                    match generic {
                        AST::token { addr } => {
                            generics.push(addr.clone())
                        },
                        AST::comma => {

                        },
                        AST::newline => {

                        }
                        _ => {
                            return Err(NP_Error::Custom { message: String::from("Unexpected token in generics!")})
                        }
                    }
                }

                result_schema.self_generics = Some((schema_len, generics));

                Ok(index + 1)
            },
            _ => {
                Ok(index)
            }
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

        if has_colons { // key: value, key: value

            let mut state = ChildItemParseState::Key;

            let mut key_str: AST_STR = Default::default();
            let mut final_args = NP_HashMap::new();
            while i < items.len() && state != ChildItemParseState::Finished {

                match state {
                    ChildItemParseState::Key => {
                        if let AST::token { addr } = items[i] {
                            key_str = addr.clone();
                            state = ChildItemParseState::Colon;
                            i += 1;
                        } else {
                            return Err(NP_Error::Custom { message: String::from("Error parsing argument key:value pairs!")})
                        }
                    },
                    ChildItemParseState::Colon => {  // colon
                        if items[i] != AST::colon {
                            return Err(NP_Error::Custom { message: String::from("Error parsing argument key:value pairs!")})
                        } else {
                            state = ChildItemParseState::Value;
                            i += 1;
                        }
                    },
                    ChildItemParseState::Value => { // value

                        match &items[i] {
                            AST::token { addr } => {
                                let token_value = addr.read(source);
                                match token_value {
                                    "true" => {
                                        final_args.set(key_str.read(source), NP_Schema_Args::TRUE)?;
                                    },
                                    "false" => {
                                        final_args.set(key_str.read(source), NP_Schema_Args::FALSE)?;
                                    },
                                    "null" => {
                                        final_args.set(key_str.read(source), NP_Schema_Args::NULL)?;
                                    },
                                    _ => {

                                    }
                                }
                            },
                            AST::number { addr } => {
                                final_args.set(key_str.read(source), NP_Schema_Args::NUMBER(addr.clone()))?;
                            },
                            AST::string { addr } => {
                                final_args.set(key_str.read(source), NP_Schema_Args::STRING(addr.clone()))?;
                            },
                            AST::square { items } => {
                                final_args.set(key_str.read(source), Self::parse_argument_groups(source, items)?)?;
                            },
                            _ => {
                                return Err(NP_Error::Custom { message: String::from("Error parsing argument key:value pairs!")})
                            }
                        }

                        state = ChildItemParseState::Comma;
                        i += 1;
                    },
                    ChildItemParseState::Comma => { // comma
                        while i < items.len() && (&items[i] == &AST::comma || &items[i] == &AST::newline) {
                            i += 1;
                        }
                        state = ChildItemParseState::Key;
                    }
                    _ => {} // other
                }
            }

            Ok(NP_Schema_Args::MAP (final_args))
        } else { // value, value, value

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
                                    },
                                    "false" => {
                                        final_args.push(NP_Schema_Args::FALSE);
                                    },
                                    "null" => {
                                        final_args.push(NP_Schema_Args::NULL);
                                    },
                                    _ => {}
                                }
                            },
                            AST::number { addr } => {
                                final_args.push(NP_Schema_Args::NUMBER (addr.clone()));
                            },
                            AST::string { addr } => {
                                final_args.push(NP_Schema_Args::STRING (addr.clone()));
                            },
                            AST::square { items } => {
                                final_args.push( Self::parse_argument_groups(source, items)?);
                            },
                            _ => {
                                return Err(NP_Error::Custom { message: String::from("Error parsing argument key:value pairs!") })
                            }
                        }

                        state = ChildItemParseState::Comma;
                        i += 1;
                    },
                    ChildItemParseState::Comma => {
                        while i < items.len() && (&items[i] == &AST::comma || &items[i] == &AST::newline) {
                            i += 1;
                        }
                        state = ChildItemParseState::Key;
                    },
                    _ => {

                    }
                }
            }

            Ok(NP_Schema_Args::LIST(final_args))
        }

    }


    fn maybe_parse_arguments(source: &str, ast: &Vec<AST>, index: usize, max_index: usize, result_schema: &mut NP_Parsed_Schema) -> Result<usize, NP_Error> {

        if index + 1 >= max_index {
            return Ok(index);
        }

        match &ast[index + 1] {
            AST::square { items} => {
                result_schema.arguments = Self::parse_argument_groups(source, items)?;
                Ok(index + 1)
            },
            _ => {
                Ok(index)
            }
        }


    }

    fn str_to_type(source: &str, token: &AST_STR, generics: &Option<(usize, Vec<AST_STR>)>, type_idx: &NP_HashMap<NP_Schema_Index>) -> Option<NP_Schema_Type> {

        let token_value = token.read(source);

        match token_value {
            "any"      => Some(NP_Schema_Type::Any { size: 1 + POINTER_SIZE } ),
            "info"     => Some(NP_Schema_Type::Info   ),
            "string"   => Some(NP_Schema_Type::String { size: POINTER_SIZE, default: Default::default(), casing: NP_String_Casing::None, max_len: None }),
            "char"     => Some(NP_Schema_Type::Char  { size: 1, default: char::from(0) }  ),
            "i8"       => Some(NP_Schema_Type::Int8 { size: 1, default: 0, min: None, max: None }   ),
            "i16"      => Some(NP_Schema_Type::Int16 { size: 2, default: 0, min: None, max: None }  ),
            "i32"      => Some(NP_Schema_Type::Int32 { size: 4, default: 0, min: None, max: None }  ),
            "i64"      => Some(NP_Schema_Type::Int64 { size: 8, default: 0, min: None, max: None }  ),
            "u8"       => Some(NP_Schema_Type::Uint8 { size: 1, default: 0, min: None, max: None }  ),
            "u16"      => Some(NP_Schema_Type::Uint16 { size: 2, default: 0, min: None, max: None } ),
            "u32"      => Some(NP_Schema_Type::Uint32 { size: 4, default: 0, min: None, max: None } ),
            "u64"      => Some(NP_Schema_Type::Uint64 { size: 8, default: 0, min: None, max: None } ),
            "f32"      => Some(NP_Schema_Type::f32 { size: 4, default: 0.0, min: None, max: None }  ),
            "f64"      => Some(NP_Schema_Type::f64 { size: 8, default: 0.0, min: None, max: None }  ),
            "dec32"    => Some(NP_Schema_Type::Dec32 { size: 4, default: 0, exp: 0, min: None, max: None }  ),
            "dec64"    => Some(NP_Schema_Type::Dec64 { size: 8, default: 0, exp: 0, min: None, max: None }  ),
            "bool"     => Some(NP_Schema_Type::Boolean { size: 1, default: false } ),
            "geo32"    => Some(NP_Schema_Type::Geo32 { size: 4, default: (0,0) }   ),
            "geo64"    => Some(NP_Schema_Type::Geo64 { size: 8, default: (0,0) }   ),
            "geo128"   => Some(NP_Schema_Type::Geo128 { size: 16, default: (0,0) } ),
            "uuid"     => Some(NP_Schema_Type::Uuid { size: 16 }  ),
            "ulid"     => Some(NP_Schema_Type::Ulid { size: 16 }  ),
            "date"     => Some(NP_Schema_Type::Date { size: 8, default: 0 }   ),
            "enum"     => Some(NP_Schema_Type::Enum { size:  1 + POINTER_SIZE, children: Default::default(), default: 0 }   ),
            "struct"   => Some(NP_Schema_Type::Struct { size: Default::default(), children: Default::default() } ),
            "Map"      => Some(NP_Schema_Type::Map { size: POINTER_SIZE } ),
            "Vec"      => Some(NP_Schema_Type::Vec { size: POINTER_SIZE, max_len: None } ),
            "Result"   => Some(NP_Schema_Type::Result { size: 1 + POINTER_SIZE } ),
            "Option"   => Some(NP_Schema_Type::Option { size: 1 + POINTER_SIZE } ),
            "Box"      => Some(NP_Schema_Type::Box { size: POINTER_SIZE } ),
            "impl"     => Some(NP_Schema_Type::Impl { children: Default::default() } ),
            "self"     => Some(NP_Schema_Type::Fn_Self { idx: 0 }),
            "Self"     => Some(NP_Schema_Type::Fn_Self { idx: 0 }),
            "tuple"    => Some(NP_Schema_Type::Tuple { size: POINTER_SIZE, children: Default::default() }  ),
            _ => {

                // is this a valid generic type?
                if let Some(these_generics) = generics {
                    for (idx, generic) in these_generics.1.iter().enumerate() {
                        if generic.read(source) == token_value {
                            return Some(NP_Schema_Type::Generic { parent_scham_addr: these_generics.0, generic_idx: idx, size: POINTER_SIZE })
                        }
                    }
                }

                // is this a valid custom type?
                if let Some(type_data) = type_idx.get(token_value) {
                    return Some(NP_Schema_Type::Custom { type_idx: type_data.data, size: POINTER_SIZE });
                }

                return None;
            }
        }
    }

    fn parse_single_type(source: &str, ast: &Vec<AST>, index: usize, depth: u16, parent_idx: usize, generics: &Option<(usize, Vec<AST_STR>)>, type_idx: &mut NP_HashMap<NP_Schema_Index>, parsed_schema: &mut Vec<NP_Parsed_Schema>) -> Result<usize, NP_Error> {

        if depth > 255 {
            return Err(NP_Error::RecursionLimit)
        }

        // find where the next newline, semicolon or comma is.  Parsing should not pass this point.
        let mut max_index = index;
        while max_index < ast.len() && ast[max_index] != AST::semicolon && ast[max_index] != AST::newline && ast[max_index] != AST::comma {
            max_index += 1;
        }

        let mut use_index = index;
        let this_ast = &ast[use_index];


        // inject placeholder schema
        let mut result_schema: NP_Parsed_Schema = NP_Parsed_Schema::default();
        let schema_len = parsed_schema.len();
        parsed_schema.push(NP_Parsed_Schema::default());

        let mut child_generics: Vec<usize> = Vec::new();

        let mut is_implicit = false;
        let mut is_struct = false;

        let mut child_items = match this_ast {
            AST::curly { items } => { // implicit struct
                result_schema.data_type = NP_Schema_Type::Struct { children: Default::default(), size: POINTER_SIZE };
                is_implicit = true;
                is_struct = true;
                Some(items)
            },
            AST::parans { items } => { // tuple type (X, Y, Z) or method (x, y) -> z
                let mut has_arrows = false;
                let mut check_index = use_index;
                while check_index < max_index {
                    if let AST::arrow = &ast[check_index] {
                        has_arrows = true;
                    }
                    check_index += 1;
                }

                if has_arrows {
                    result_schema.data_type = NP_Schema_Type::Method { args: Default::default(), returns: 0 };
                } else {
                    is_implicit = true;
                    result_schema.data_type = NP_Schema_Type::Tuple { children: Default::default(), size: 0 };
                    use_index = Self::maybe_parse_title(ast, use_index, max_index, &mut result_schema);
                    use_index = Self::maybe_parse_arguments(source, ast, use_index, max_index, &mut result_schema)?;
                }

                Some(items)
            },
            AST::square { items } => { // array type [X; 32]
                result_schema.data_type = NP_Schema_Type::Array { len: 0, size: 0 };
                use_index = Self::maybe_parse_title(ast, use_index, max_index, &mut result_schema);
                use_index = Self::maybe_parse_arguments(source, ast, use_index, max_index, &mut result_schema)?;
                Some(items)
            },
            AST::token { addr } => { // standard named type

                // handle types with generic parameters like Vec<u32>
                if ast.len() > use_index + 1 {
                    if let AST::xml { items } = &ast[use_index + 1] {

                        if addr.read(source) != "impl" { // ignore impl generics
                            let mut i:usize = 0;
                            while i < items.len() {
                                if items[i] != AST::comma && items[i] != AST::newline {
                                    child_generics.push(parsed_schema.len());
                                    i = Self::parse_single_type(&source, items, i, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                }
                                i += 1;
                            }
                        }

                        use_index += 1;
                    }
                }

                if child_generics.len() > 0 {
                    result_schema.use_generics = Some(child_generics);
                }

                use_index = Self::maybe_parse_title(ast, use_index, max_index, &mut result_schema);
                use_index = Self::maybe_parse_generics(ast, use_index, max_index, schema_len, &mut result_schema)?;
                use_index = Self::maybe_parse_arguments(source, ast, use_index, max_index, &mut result_schema)?;

                if let Some(data_type) = Self::str_to_type(source, addr, &generics, &type_idx) {
                    result_schema.data_type = data_type;
                    if let NP_Schema_Type::Struct { .. } = &result_schema.data_type {
                        is_struct = true;
                    }
                } else { // no type found!
                    let mut err = String::from("Unknown type found!: ");
                    err.push_str(addr.read(source));
                    return Err(NP_Error::Custom { message: err});
                }

                None
            },
            _ => {
                return Err(NP_Error::Custom { message: String::from("Unexpected value in parsing AST!")})
            }
        };

        // set type index
        if let Some(title) = result_schema.name {
            if depth == 0 {
                if let NP_Schema_Type::Impl { .. } = result_schema.data_type { // impl block

                    let index_data = if let Some(index_data) = type_idx.get(title.read(source)) {
                        index_data.clone()
                    } else {
                        return Err(NP_Error::Custom { message: String::from("impl block before data declaration!")})
                    };

                    type_idx.set(title.read(source), NP_Schema_Index {
                        data: index_data.data,
                        methods: Some(schema_len)
                    })?;

                } else { // any other type
                    type_idx.set(title.read(source), NP_Schema_Index { data: schema_len, methods: None })?;
                }
            }
        }


        if is_struct && max_index > use_index + 1 {
            match &ast[use_index + 1] {
                AST::parans { .. } => { // actually a tuple type!
                    result_schema.data_type = NP_Schema_Type::Tuple { children: Vec::new(), size: 0 };
                },
                _ => { }
            }
        }

        // type generics not allowed on nested types
        if depth > 0 && result_schema.self_generics != None {
            return Err(NP_Error::Custom { message: String::from("Nested types cannot have generic arguments!")})
        }

        match &mut result_schema.data_type {
            NP_Schema_Type::None => { /* nothing to do */ }
            NP_Schema_Type::Any  { .. } => { /* nothing to do */ }
            NP_Schema_Type::Info => { /* nothing to do */ }
            NP_Schema_Type::String { default, casing, max_len, .. } => {

                if let NP_Schema_Args::MAP (args_map ) = &result_schema.arguments {
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
            NP_Schema_Type::Char { default, .. } => {
                if let NP_Schema_Args::MAP (args_map) = &result_schema.arguments {
                    if let Some(NP_Schema_Args::STRING (data) ) = args_map.get("default") {
                        if let Some(char) = data.read(source).chars().next() {
                            *default = char;
                        }
                    }
                }
            }
            NP_Schema_Type::Int8 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, i8, default, min, max);
            }
            NP_Schema_Type::Int16 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, i16, default, min, max);
            }
            NP_Schema_Type::Int32 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, i32, default, min, max);
            }
            NP_Schema_Type::Int64 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, i64, default, min, max);
            }
            NP_Schema_Type::Uint8 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, u8, default, min, max);
            }
            NP_Schema_Type::Uint16 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, u16, default, min, max);
            }
            NP_Schema_Type::Uint32 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, u32, default, min, max);
            }
            NP_Schema_Type::Uint64 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, u64, default, min, max);
            }
            NP_Schema_Type::f32 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, f32, default, min, max);
            }
            NP_Schema_Type::f64 { default, min, max, .. } => {
                let args = &result_schema.arguments;
                schema_number!(source, args, f64, default, min, max);
            }
            NP_Schema_Type::Dec32 { default, exp, min, max, .. } => {

                if let NP_Schema_Args::MAP (args_map) =  &result_schema.arguments {
                    let mut multiple: f64 = 10.0;
                    let mut is_exp_neg = false;

                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("exp") {
                        if let Ok(value) = data.read(source).parse::<i16>()  {
                            *exp = value;
                            if value < 0 {
                                is_exp_neg = true;
                            }
                            let mut index: i16 = 1;
                            while index < value.abs() {
                                multiple *= 10.0;
                                index += 1;
                            }
                        }
                    }

                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("default")  {
                        if let Ok(value) = data.read(source).parse::<f64>()  {
                            *default = (if is_exp_neg {
                                value / multiple
                            } else {
                                value * multiple
                            }) as i32;
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("min") {
                        if let Ok(value) = data.read(source).parse::<f64>()  {
                            *min = Some((if is_exp_neg {
                                value / multiple
                            } else {
                                value * multiple
                            }) as i32);
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("max") {
                        if let Ok(value) = data.read(source).parse::<f64>()  {
                            *max = Some((if is_exp_neg {
                                value / multiple
                            } else {
                                value * multiple
                            }) as i32);
                        }
                    }

                }
            }
            NP_Schema_Type::Dec64 { default, exp, min, max, .. } => {

                if let NP_Schema_Args::MAP (args_map) =  &result_schema.arguments {
                    let mut multiple: f64 = 10.0;
                    let mut is_exp_neg = false;

                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("exp") {
                        if let Ok(value) = data.read(source).parse::<i16>()  {
                            *exp = value;
                            if value < 0 {
                                is_exp_neg = true;
                            }
                            let mut index: i16 = 1;
                            while index < value.abs() {
                                multiple *= 10.0;
                                index += 1;
                            }
                        }
                    }

                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("default")  {
                        if let Ok(value) = data.read(source).parse::<f64>()  {
                            *default = (if is_exp_neg {
                                value / multiple
                            } else {
                                value * multiple
                            }) as i64;
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("min") {
                        if let Ok(value) = data.read(source).parse::<f64>()  {
                            *min = Some((if is_exp_neg {
                                value / multiple
                            } else {
                                value * multiple
                            }) as i64);
                        }
                    }
                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("max") {
                        if let Ok(value) = data.read(source).parse::<f64>()  {
                            *max = Some((if is_exp_neg {
                                value / multiple
                            } else {
                                value * multiple
                            }) as i64);
                        }
                    }

                }
            }
            NP_Schema_Type::Boolean { default, .. } => {
                if let NP_Schema_Args::MAP (args_map) = &result_schema.arguments {
                    if let Some(NP_Schema_Args::TRUE) = args_map.get("default") {
                        *default = true;
                    }
                    if let Some(NP_Schema_Args::FALSE) = args_map.get("default") {
                        *default = false;
                    }
                }
            }
            NP_Schema_Type::Geo32 { default, .. } => {
                let args = &result_schema.arguments;
                schema_geo!(source, args, i16, default, 100f64);
            }
            NP_Schema_Type::Geo64 { default, .. } => {
                let args = &result_schema.arguments;
                schema_geo!(source, args, i32, default, 10000000f64);
            }
            NP_Schema_Type::Geo128 { default, .. } => {
                let args = &result_schema.arguments;
                schema_geo!(source, args, i64, default, 1000000000f64);
            }
            NP_Schema_Type::Uuid { .. } => {}
            NP_Schema_Type::Ulid { .. } => {}
            NP_Schema_Type::Date { default, ..  } => {
                if let NP_Schema_Args::MAP (args_map) = &result_schema.arguments {
                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("default") {
                        if let Ok(value) = data.read(source).parse::<u64>() {
                            *default = value;
                        }
                    }
                }
            }
            NP_Schema_Type::Enum { children, default, .. } => {
                let (next_index, children_items_ast) = Self::maybe_parse_children(ast, use_index, max_index, false);
                use_index = next_index;
                child_items = children_items_ast;

                if let Some(children_ast) = child_items {
                    let mut parse_idx: usize = 0;
                    let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Key;


                    while parse_idx < children_ast.len() {

                        match parse_state {
                            ChildItemParseState::Key => {
                                if let AST::token { addr } = &children_ast[parse_idx] {
                                    key_ast = addr.clone();

                                    if parse_idx + 1 >= children_ast.len() {
                                        children.set(key_ast.read(source), None)?;
                                        parse_state = ChildItemParseState::Finished;
                                        parse_idx += 1;
                                    } else {
                                        parse_state = ChildItemParseState::Colon;
                                        parse_idx += 1;
                                    }

                                } else {
                                    return Err(NP_Error::Custom { message: String::from("Error parsing enum child items!")});
                                }
                            },
                            ChildItemParseState::Colon => {

                                match &children_ast[parse_idx] {
                                    AST::comma => {
                                        // has no child types
                                        children.set(key_ast.read(source), None)?;
                                        parse_state = ChildItemParseState::Comma;
                                        parse_idx += 1;
                                    },
                                    AST::parans { .. } => {
                                        parse_state = ChildItemParseState::Value;
                                    },
                                    AST::curly { .. } => {
                                        parse_state = ChildItemParseState::Value;
                                    },
                                    AST::newline => {
                                        // has no child types
                                        children.set(key_ast.read(source), None)?;
                                        parse_state = ChildItemParseState::Comma;
                                        parse_idx += 1;
                                    },
                                    _ => {
                                        return Err(NP_Error::Custom { message: String::from("Error parsing enum child items!")});
                                    }
                                }

                            }
                            ChildItemParseState::Value => {

                                let schema_loc = parsed_schema.len();
                                children.set(key_ast.read(source), Some(schema_loc))?;

                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                                } else {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            },
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {

                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            },
                                            AST::newline => {
                                                parse_idx += 1;
                                            },
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

                    if let NP_Schema_Args::MAP ( data ) = &result_schema.arguments {
                        if let Some(NP_Schema_Args::STRING ( data )) = data.get("default") {
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
                                return Err(NP_Error::Custom { message: String::from("Enum default cannot contain properties!")})
                            }
                        }
                    } else {
                        return Err(NP_Error::Custom { message: String::from("Enums require default property!")})
                    }

                } else {
                    return Err(NP_Error::Custom { message: String::from("Missing enum children declaration!")})
                }

            }
            NP_Schema_Type::Struct { children, size } => {
                if is_implicit == false {
                    let (next_index, children_ast_items) = Self::maybe_parse_children(ast, use_index, max_index, false);
                    use_index = next_index;
                    child_items = children_ast_items;
                }

                let mut running_size: u32 = 0;

                if let Some(children_ast) = child_items {
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
                                    return Err(NP_Error::Custom { message: String::from("Error parsing struct child items!")});
                                }
                            },
                            ChildItemParseState::Colon => {
                                if let AST::colon = &children_ast[parse_idx] {
                                    parse_state = ChildItemParseState::Value;
                                    parse_idx += 1;
                                } else {
                                    return Err(NP_Error::Custom { message: String::from("Error parsing struct child items!")});
                                }
                            }
                            ChildItemParseState::Value => {
                                let schema_loc = parsed_schema.len();
                                children.set(key_ast.read(source), schema_loc)?;
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                                } else {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                }

                                parsed_schema[schema_loc].offset = running_size as usize;
                                running_size += parsed_schema[schema_loc].data_type.type_info().2;

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            },
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            },
                                            AST::newline => {
                                                parse_idx += 1;
                                            },
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
                    return Err(NP_Error::Custom { message: String::from("Missing struct children declaration!")})
                }

                *size = running_size;

            }
            NP_Schema_Type::Map { .. } => { /* nothing to do */ },
            NP_Schema_Type::Vec { max_len, .. } => {
                if let NP_Schema_Args::MAP (args_map) = &result_schema.arguments {
                    if let Some(NP_Schema_Args::NUMBER ( data )) = args_map.get("max_len") {
                        if let Ok(value) = data.read(source).parse::<u64>() {
                            *max_len = Some(value);
                        }
                    }
                }
            },
            NP_Schema_Type::Result { .. } => { /* nothing to do */ },
            NP_Schema_Type::Option  { .. } => { /* nothing to do */ },
            NP_Schema_Type::Box     { .. } => { /* nothing to do */ },
            NP_Schema_Type::Fn_Self { idx } => {
                *idx = parent_idx;
            },
            NP_Schema_Type::Array { len, size  } => {

                let running_size: u32;

                if let Some(children) = child_items {
                    let mut parse_idx: usize = 0;
                    let child_type = parsed_schema.len();
                    if depth == 0 {
                        parse_idx = Self::parse_single_type(source, children, parse_idx, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                    } else {
                        parse_idx = Self::parse_single_type(source, children, parse_idx, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                    }
                    parse_idx +=1;

                    result_schema.use_generics = Some(vec![child_type]);

                    if let AST::semicolon = &children[parse_idx] {
                        parse_idx += 1;
                    } else {
                        return Err(NP_Error::Custom { message: String::from("Error parsing array type!") })
                    }

                    if let AST::number { addr } = &children[parse_idx] {
                        if let Ok(length) = addr.read(source).parse::<usize>() {
                            *len = length;
                        } else {
                            return Err(NP_Error::Custom { message: String::from("Error parsing array type!") })
                        }
                    } else {
                        return Err(NP_Error::Custom { message: String::from("Error parsing array type!") })
                    }

                    running_size = (*len * (parsed_schema[child_type].data_type.type_info().2 as usize)) as u32;

                } else {
                    return Err(NP_Error::Custom { message: String::from("Missing array items!")})
                }

                *size = running_size;
            }
            NP_Schema_Type::Tuple { children, size } => {

                let mut running_size: u32 = 0;

                if is_implicit == false {
                    let (next_index, parsed_children) = Self::maybe_parse_children(ast, use_index, max_index, true);
                    use_index = next_index;
                    child_items = parsed_children;
                }

                if let Some(children_ast) = child_items {
                    let mut parse_idx: usize = 0;
                    // let mut key_ast = AST_STR::default();
                    let mut parse_state = ChildItemParseState::Value;

                    while parse_idx < children_ast.len() {

                        match parse_state {
                            ChildItemParseState::Key => { /* no keys here */ },
                            ChildItemParseState::Colon => { /* no colons here */ }
                            ChildItemParseState::Value => {
                                let schema_loc = parsed_schema.len();
                                children.push(schema_loc);
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                                } else {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, parent_idx,&generics, type_idx, parsed_schema)?;
                                }

                                parsed_schema[schema_loc].offset = running_size as usize;
                                running_size += parsed_schema[schema_loc].data_type.type_info().2;

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            },
                            ChildItemParseState::Comma => {

                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            },
                                            AST::newline => {
                                                parse_idx += 1;
                                            },
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
                    return Err(NP_Error::Custom { message: String::from("Missing tuple children declaration!")})
                }

                *size = running_size;
            }
            NP_Schema_Type::Impl { children } => {
                let (next_index, children_ast_items) = Self::maybe_parse_children(ast, use_index, max_index, false);
                use_index = next_index;
                child_items = children_ast_items;

                if let Some(children_ast) = child_items {
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
                                    return Err(NP_Error::Custom { message: String::from("Error parsing impl child items!")});
                                }
                            },
                            ChildItemParseState::Colon => { /* no colons here */ }
                            ChildItemParseState::Value => {
                                children.set(key_ast.read(source), parsed_schema.len())?;
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, schema_len,&result_schema.self_generics, type_idx, parsed_schema)?;
                                } else {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            },
                            ChildItemParseState::Comma => {

                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            },
                                            AST::newline => {
                                                parse_idx += 1;
                                            },
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
                    return Err(NP_Error::Custom { message: String::from("Missing impl children declaration!")})
                }
            }
            NP_Schema_Type::Method { args, returns } => {

                // parse args
                if let Some(children_ast) = child_items {
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
                                    return Err(NP_Error::Custom { message: String::from("Error parsing method args!")});
                                }
                            },
                            ChildItemParseState::Colon => {
                                match &children_ast[parse_idx] {
                                    AST::colon => { // named param
                                        parse_state = ChildItemParseState::Value;
                                        parse_idx += 1;
                                    },
                                    AST::comma => { // anonymous param
                                        // args.push((None, parsed_schema.len()));
                                        args.set("self", parsed_schema.len())?;
                                        if depth == 0 {
                                            parse_idx = Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                                        } else {
                                            parse_idx = Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                        }

                                        parse_state = ChildItemParseState::Comma;
                                        parse_idx += 1;
                                    },
                                    _ => {
                                        return Err(NP_Error::Custom { message: String::from("Error parsing struct child items!")});
                                    }
                                }
                            }
                            ChildItemParseState::Value => {
                                args.set(key_ast.read(source), parsed_schema.len())?;
                                // args.push((Some(String::from(key_ast.read(source))), parsed_schema.len()));
                                if depth == 0 {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                                } else {
                                    parse_idx = Self::parse_single_type(source, children_ast, parse_idx, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                                }

                                parse_state = ChildItemParseState::Comma;
                                parse_idx += 1;
                            },
                            ChildItemParseState::Comma => {
                                let mut loop_max: u8 = 0;

                                while let ChildItemParseState::Comma = parse_state {
                                    if children_ast.len() > parse_idx {
                                        match &children_ast[parse_idx] {
                                            AST::comma => {
                                                parse_idx += 1;
                                            },
                                            AST::newline => {
                                                parse_idx += 1;
                                            },
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

                    // last item in args was anonymous arg
                    if let ChildItemParseState::Colon = parse_state {
                        args.set("self", parsed_schema.len())?;
                        // args.push((None, parsed_schema.len()));
                        if depth == 0 {
                            Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                        } else {
                            Self::parse_single_type(source, children_ast, parse_idx - 1, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                        }

                    }
                }


                use_index += 1;


                // parse return value
                if let AST::arrow = &ast[use_index] {
                    use_index += 1;
                } else {
                    return Err(NP_Error::Custom { message: String::from("Missing arrow from method declaration!")});
                }

                *returns = parsed_schema.len();

                if depth == 0 {
                    use_index = Self::parse_single_type(source, ast, use_index, depth + 1, schema_len, &result_schema.self_generics, type_idx, parsed_schema)?;
                } else {
                    use_index = Self::parse_single_type(source, ast, use_index, depth + 1, parent_idx, &generics, type_idx, parsed_schema)?;
                }

                use_index += 1;

            }
            NP_Schema_Type::Generic { .. } => { /* nothing to do */ }
            NP_Schema_Type::Custom { .. } => { /* nothing to do */ }
        }

        Self::maybe_error_on_generics(&result_schema)?;


        // set ID
        if let NP_Schema_Args::MAP ( data ) = &result_schema.arguments {
            if let Some(id) = data.get("id") {
                if let NP_Schema_Args::NUMBER(  data ) = id {
                    if let Ok(id_num) = data.read(source).parse::<u16>() {
                        result_schema.id = Some(id_num);
                    }
                }
            }
        }

        if depth == 0 {

            if result_schema.data_type != NP_Schema_Type::Info {
                if None == result_schema.id {
                    if let NP_Schema_Type::Impl { .. } = &result_schema.data_type {

                    } else {
                        return Err(NP_Error::Custom { message: String::from("All top level types must have an id property!")})
                    }
                }
                if None == result_schema.name {
                    return Err(NP_Error::Custom { message: String::from("All top level types must have a name!")})
                }
            } else {
                type_idx.set("__info", NP_Schema_Index { data: schema_len, methods: None})?;
            }
        }

        if result_schema.data_type == NP_Schema_Type::None {
            return Err(NP_Error::Custom { message: String::from("No valid data type found!")})
        }

        // set result schema
        parsed_schema[schema_len] = result_schema;

        Ok(use_index)
    }

    pub fn read_ast_str(&self, ast_str: AST_STR) -> &str {
        ast_str.read_bytes(&self.source.as_slice())
    }
}