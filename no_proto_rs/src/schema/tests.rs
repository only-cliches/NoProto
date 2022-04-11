

#[cfg(test)]
mod schema_tests {

    use crate::schema::schema_args::NP_Args;
    use crate::error::NP_Error;
    use crate::schema::{NP_Schema, NP_Schema_Index, POINTER_SIZE, NP_Schema_Type, NP_String_Casing, NP_Parsed_Generics};
    use crate::schema::ast_parser::{AST_STR, AST};
    use alloc::prelude::v1::{Vec, String};
    use crate::map::NP_HashMap;

    #[test]
    fn empty_parse_1() -> Result<(), NP_Error> {

        let schema = r##""##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 0);

        Ok(())
    }

    #[test]
    fn any_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            any myType [id: 0, other: "hello"]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(0), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Any);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("0")));
        assert_eq!(parsed.schemas[0].arguments.query("other", schema), Some(NP_Args::STRING("hello")));
        assert_eq!(parsed.schemas[0].id, Some(0));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn info_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            info [
                name: "Jeb Kermin",
                id: 200,
                email: "jeb@ksp.org",
                some_option: true,
                another_option: false,
                no_value: null,
                colors: [
                    "red",
                    "orange",
                    "green"
                ],
                meta: [
                    fav_sport: "golf",
                    fav_color: "green",
                    monthly_income: 200.58,
                    nested_list: [1, 2, 3, 4]
                ],
                more_meta: [key: "value", key2: 500]
            ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("__info"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(0), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Info);
        assert_eq!(parsed.schemas[0].id, Some(200));
        assert_eq!(parsed.schemas[0].arguments.query("name", schema), Some(NP_Args::STRING("Jeb Kermin") ));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("200") ));
        assert_eq!(parsed.schemas[0].arguments.query("email", schema), Some(NP_Args::STRING("jeb@ksp.org")));
        assert_eq!(parsed.schemas[0].arguments.query("some_option", schema), Some(NP_Args::TRUE));
        assert_eq!(parsed.schemas[0].arguments.query("another_option", schema), Some(NP_Args::FALSE));
        assert_eq!(parsed.schemas[0].arguments.query("no_value", schema), Some(NP_Args::NULL));
        assert_eq!(parsed.schemas[0].arguments.query("colors.0", schema), Some(NP_Args::STRING("red") ));
        assert_eq!(parsed.schemas[0].arguments.query("colors.1", schema), Some(NP_Args::STRING("orange") ));
        assert_eq!(parsed.schemas[0].arguments.query("colors.2", schema), Some(NP_Args::STRING("green") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.fav_sport", schema), Some(NP_Args::STRING ("golf") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.fav_color", schema), Some(NP_Args::STRING ("green") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.monthly_income", schema), Some(NP_Args::NUMBER("200.58") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.nested_list.0", schema), Some(NP_Args::NUMBER("1") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.nested_list.1", schema), Some(NP_Args::NUMBER("2") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.nested_list.2", schema), Some(NP_Args::NUMBER("3") ));
        assert_eq!(parsed.schemas[0].arguments.query("meta.nested_list.3", schema), Some(NP_Args::NUMBER("4") ));
        assert_eq!(parsed.schemas[0].arguments.query("more_meta.key", schema), Some(NP_Args::STRING("value") ));
        assert_eq!(parsed.schemas[0].arguments.query("more_meta.key2", schema), Some(NP_Args::NUMBER("500") ));

        Ok(())
    }

    #[test]
    fn string_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            string myType [id: 0]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(0), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::String { default: AST_STR { start: 0, end: 0 }, casing: NP_String_Casing::None, max_len: None });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("0")));
        assert_eq!(parsed.schemas[0].id, Some(0));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn string_parse_2() -> Result<(), NP_Error> {

        let schema = r##"
            string myType [id: 0, default: "hello", max_len: 20, uppercase: true]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(0), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::String { default: AST_STR { start: 45, end: 50 }, casing: NP_String_Casing::Uppercase, max_len: Some(20) });
        if let NP_Schema_Type::String { default, .. } = parsed.schemas[0].data_type {
            assert_eq!(default.read(schema), "hello");
        } else {
            assert!(false);
        }
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("0")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::STRING("hello")));
        assert_eq!(parsed.schemas[0].id, Some(0));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn string_parse_3() -> Result<(), NP_Error> {

        let schema = r##"
            string myType [id: 0, default: "hello", max_len: 50, lowercase: true]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(0), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::String { default: AST_STR { start: 45, end: 50 }, casing: NP_String_Casing::Lowercase, max_len: Some(50) });
        if let NP_Schema_Type::String { default, .. } = parsed.schemas[0].data_type {
            assert_eq!(default.read(schema), "hello");
        } else {
            assert!(false);
        }
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("0")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::STRING("hello")));
        assert_eq!(parsed.schemas[0].id, Some(0));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn char_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            char myType [id: 0, default: "j"]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(0), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Char { default: 'j' });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("0")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::STRING("j")));
        assert_eq!(parsed.schemas[0].id, Some(0));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn i8_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            i8 myType [id: 2, default: 20, max: 10, min: -50]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Int8 { default: 20, max: Some(10), min: Some(-50) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("10")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("-50")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn i16_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            i16 myType [id: 2, default: 20, max: 10, min: -50]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Int16 { default: 20, max: Some(10), min: Some(-50) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("10")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("-50")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn i32_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            i32 myType [id: 2, default: 20, max: 10, min: -50]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Int32 { default: 20, max: Some(10), min: Some(-50) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("10")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("-50")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn i64_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            i64 myType [id: 2, default: 20, max: 10, min: -50]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Int64 { default: 20, max: Some(10), min: Some(-50) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("10")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("-50")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn u8_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            u8 myType [id: 2, default: 20, max: 100, min: 5]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Uint8 { default: 20, max: Some(100), min: Some(5) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn u16_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            u16 myType [id: 2, default: 20, max: 100, min: 5]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Uint16 { default: 20, max: Some(100), min: Some(5) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn u32_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            u32 myType [id: 2, default: 20, max: 100, min: 5]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Uint32 { default: 20, max: Some(100), min: Some(5) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn u64_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            u64 myType [id: 2, default: 20, max: 100, min: 5]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Uint64 { default: 20, max: Some(100), min: Some(5) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn f32_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            f32 myType [id: 2, default: 20, max: 100.2, min: 5.1]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::f32 { default: 20.0, max: Some(100.2), min: Some(5.1) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100.2")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5.1")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn f64_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            f64 myType [id: 2, default: 20, max: 100.2, min: 5.5]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::f64 { default: 20.0, max: Some(100.2), min: Some(5.5) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100.2")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5.5")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn dec32_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            dec32 myType [id: 2, default: 25, max: 100, min: 5, exp: 2]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Dec32 { default: 2500, exp: 2, max: Some(10000), min: Some(500) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("25")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("100")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5")));
        assert_eq!(parsed.schemas[0].arguments.query("exp", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn dec64_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            dec64 myType [id: 2, default: 20392039, max: 1293838, min: 5206, exp: -2]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Dec64 { default: 203920, exp: -2, max: Some(12938), min: Some(52) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::NUMBER("20392039")));
        assert_eq!(parsed.schemas[0].arguments.query("max", schema), Some(NP_Args::NUMBER("1293838")));
        assert_eq!(parsed.schemas[0].arguments.query("min", schema), Some(NP_Args::NUMBER("5206")));
        assert_eq!(parsed.schemas[0].arguments.query("exp", schema), Some(NP_Args::NUMBER("-2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn bool_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            bool myType [id: 2, default: false]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Boolean { default:false });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default", schema), Some(NP_Args::FALSE));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn geo32_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            geo32 myType [id: 2, default: [lat: 200.29, lng: 59.20]]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Geo32 { default:(20029, 5920) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default.lat", schema), Some(NP_Args::NUMBER("200.29")));
        assert_eq!(parsed.schemas[0].arguments.query("default.lng", schema), Some(NP_Args::NUMBER("59.20")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn geo64_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            geo64 myType [id: 2, default: [lat: 200.29, lng: 59.20]]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Geo64 { default:(2002900000, 592000000) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default.lat", schema), Some(NP_Args::NUMBER("200.29")));
        assert_eq!(parsed.schemas[0].arguments.query("default.lng", schema), Some(NP_Args::NUMBER("59.20")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn geo128_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            geo128 myType [id: 2, default: [lat: 200.29, lng: 59.20]]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Geo128 { default: (200290000000, 59200000000) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("default.lat", schema), Some(NP_Args::NUMBER("200.29")));
        assert_eq!(parsed.schemas[0].arguments.query("default.lng", schema), Some(NP_Args::NUMBER("59.20")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn uuid_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            uuid myType [id: 2]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Uuid);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn ulid_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            ulid myType [id: 2]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Ulid);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn map_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            Map<string> myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 2);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Map);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[1].name, None);


        Ok(())
    }

    #[test]
    fn vec_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            Vec<string> myType [ id: 2, max_len: 20 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 2);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Vec { max_len: Some(20) });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].arguments.query("max_len", schema), Some(NP_Args::NUMBER("20")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[1].name, None);


        Ok(())
    }

    #[test]
    fn result_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            Result<u32, string> myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Result);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1, 2]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });
        assert_eq!(parsed.schemas[1].name, None);
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].name, None);


        Ok(())
    }

    #[test]
    fn option_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            Option<string> myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 2);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Option);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[1].name, None);


        Ok(())
    }

    #[test]
    fn array_parse_1() -> Result<(), NP_Error> {

        let schema = r##"
            [string; 89] myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 2);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Array { len: 89 });
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[1].name, None);


        Ok(())
    }

    #[test]
    fn nested_opts_1() -> Result<(), NP_Error> {

        let schema = r##"
            Result<u32 [opt: true], string [max_len: 20]> myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Result);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1, 2]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });
        assert_eq!(parsed.schemas[1].name, None);
        assert_eq!(parsed.schemas[1].arguments.query("opt", schema), Some(NP_Args::TRUE));
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: Some(20) });
        assert_eq!(parsed.schemas[2].name, None);
        assert_eq!(parsed.schemas[2].arguments.query("max_len", schema), Some(NP_Args::NUMBER("20")));


        Ok(())
    }

    #[test]
    fn nested_opts_2() -> Result<(), NP_Error> {

        let schema = r##"
            Result<u32 customName [opt: true], string anotherName [max_len: 20]> myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Result);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1, 2]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });
        assert_eq!(parsed.schemas[1].arguments.query("opt", schema), Some(NP_Args::TRUE));
        assert_eq!(parsed.schemas[1].name.unwrap().read(schema), "customName");
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: Some(20) });
        assert_eq!(parsed.schemas[2].name.unwrap().read(schema), "anotherName");
        assert_eq!(parsed.schemas[2].arguments.query("max_len", schema), Some(NP_Args::NUMBER("20")));


        Ok(())
    }

    #[test]
    fn nested_opts_3() -> Result<(), NP_Error> {

        let schema = r##"
            Result<u32 customName, string anotherName> myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].data_type, NP_Schema_Type::Result);
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Types(vec![1, 2]));
        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });
        assert_eq!(parsed.schemas[1].name.unwrap().read(schema), "customName");
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].name.unwrap().read(schema), "anotherName");


        Ok(())
    }

    #[test]
    fn struct_test_1() -> Result<(), NP_Error> {

        let schema = r##"
            struct myType [ id: 2 ] {
                username: string,
                email: string
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, 1),
                    "email" => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");


        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });


        Ok(())
    }

    #[test]
    fn struct_test_2() -> Result<(), NP_Error> {

        let schema = r##"
            struct myType [ id: 2 ] {
                username: string [max_len: 20, uppercase: true],
                email: string namedType [max_len: 50]
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, 1),
                    "email" => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");


        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: NP_String_Casing::Uppercase, max_len: Some(20) });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: Some(50) });
        assert_eq!(parsed.schemas[2].name.unwrap().read(schema), "namedType");

        Ok(())
    }

    #[test]
    fn enum_test_1() -> Result<(), NP_Error> {

        let schema = r##"
            enum myType [ id: 2 , default: "username" ] {
                username,
                email
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Simple_Enum { children, default } = &parsed.schemas[0].data_type {
            assert_eq!(*default, Some(0));
            assert_eq!(children, &vec!["username", "email"]);
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        Ok(())
    }

    #[test]
    fn enum_test_2() -> Result<(), NP_Error> {

        let schema = r##"
            enum myType [ id: 2 , default: "email" ] {
                username { data: string },
                email
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Enum { children, default } = &parsed.schemas[0].data_type {
            assert_eq!(*default, Some(1));
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, Some(1)),
                    "email" => assert_eq!(*value, None),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        if let NP_Schema_Type::Struct {  children } = &parsed.schemas[1].data_type {
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["data"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "data" => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        Ok(())
    }

    #[test]
    fn enum_test_3() -> Result<(), NP_Error> {

        let schema = r##"
            enum myType [ id: 2 , default: "email" ] {
                username (string),
                email
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Enum { children, default } = &parsed.schemas[0].data_type {
            assert_eq!(*default, Some(1));
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, Some(1)),
                    "email" => assert_eq!(*value, None),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        if let NP_Schema_Type::Tuple { children } = &parsed.schemas[1].data_type {
            assert_eq!(children.len(), 1);
            for (key, value) in children.iter().enumerate() {
                match key {
                    0 => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });


        Ok(())
    }


    #[test]
    fn enum_test_4() -> Result<(), NP_Error> {

        let schema = r##"
            enum myType [ id: 2 ] {
                username,
                email
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 1);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Simple_Enum {children, default } = &parsed.schemas[0].data_type {
            assert_eq!(*default, None);
            assert_eq!(children, &vec!["username", "email"]);
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");


        Ok(())
    }

    #[test]
    fn tuple_test_1() -> Result<(), NP_Error> {

        let schema = r##"
            (string, u32) myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Tuple { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.len(), 2);
            for (key, value) in children.iter().enumerate() {
                match key {
                    0 => assert_eq!(*value, 1),
                    1 => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });

        Ok(())
    }

    #[test]
    fn tuple_test_2() -> Result<(), NP_Error> {

        let schema = r##"
            struct myType [ id: 2 ] (string, u32)
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Tuple { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.len(), 2);
            for (key, value) in children.iter().enumerate() {
                match key {
                    0 => assert_eq!(*value, 1),
                    1 => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });

        Ok(())
    }

    #[test]
    fn tuple_test_3() -> Result<(), NP_Error> {

        let schema = r##"
            (string [default: "hello"], u32 [max: 2000]) myType [ id: 2 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 3);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Tuple { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.len(), 2);
            for (key, value) in children.iter().enumerate() {
                match key {
                    0 => assert_eq!(*value, 1),
                    1 => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: AST_STR { start: 32, end: 37 }, casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: Some(2000), min: None });

        Ok(())
    }

    #[test]
    fn nesting_test_1() -> Result<(), NP_Error> {

        let schema = r##"
            struct myType [ id: 2 ] {
                username: string,
                email: string,
                address: {
                    street: string,
                    city: string,
                    zip: string
                },
                primary_key: (string, u32, (uuid, string), struct {
                    key: string,
                    value: bool
                })
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        assert_eq!(parsed.schemas.len(), 16);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email", "address", "primary_key"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, 1),
                    "email" => assert_eq!(*value, 2),
                    "address" => assert_eq!(*value, 3),
                    "primary_key" => assert_eq!(*value, 7),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");

        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[3].data_type {

            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["street", "city", "zip"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "street" => assert_eq!(*value, 4),
                    "city" => assert_eq!(*value, 5),
                    "zip" => assert_eq!(*value, 6),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[4].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[5].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[6].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        if let NP_Schema_Type::Tuple { children } = &parsed.schemas[7].data_type {

            assert_eq!(children.len(), 4);
            for (key, value) in children.iter().enumerate() {
                match key {
                    0 => assert_eq!(*value, 8),
                    1 => assert_eq!(*value, 9),
                    2 => assert_eq!(*value, 10),
                    3 => assert_eq!(*value, 13),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[8].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[9].data_type, NP_Schema_Type::Uint32 { default: Default::default(), min: None, max: None });

        if let NP_Schema_Type::Tuple { children } = &parsed.schemas[10].data_type {
            assert_eq!(children.len(), 2);
            for (key, value) in children.iter().enumerate() {
                match key {
                    0 => assert_eq!(*value, 11),
                    1 => assert_eq!(*value, 12),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[11].data_type, NP_Schema_Type::Uuid);
        assert_eq!(parsed.schemas[12].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[13].data_type {

            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["key", "value"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "key" => assert_eq!(*value, 14),
                    "value" => assert_eq!(*value, 15),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[14].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[15].data_type, NP_Schema_Type::Boolean { default: Default::default() });

        Ok(())
    }

    #[test]
    fn generic_test_1() -> Result<(), NP_Error> {

        let schema = r##"
            struct myType<X, Y> [ id: 2 ] {
                username: X,
                email: Y,
                password: string
            }

            myType<u32, i64> anotherType [ id: 3 ]

            myType<Vec<u32>, i64> crazyType [ id: 4 ]
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);

        // unsafe { core::str::from_utf8_unchecked(&parsed.to_bytes()?) }
        // &parsed.to_bytes()?
        // println!("{:?} {} {}", &parsed.to_bytes()?, parsed.to_bytes()?.len(), schema.len());

        assert_eq!(parsed.schemas.len(), 11);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: None }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));
        assert_eq!(parsed.schemas[0].generics, NP_Parsed_Generics::Arguments(0, vec![AST_STR { start: 27, end: 28 }, AST_STR { start: 30, end: 31 }]));

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[0].data_type {

            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email", "password"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, 1),
                    "email" => assert_eq!(*value, 2),
                    "password" => assert_eq!(*value, 3),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");


        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::Generic { parent_scham_addr: 0, generic_idx: 0 });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::Generic { parent_scham_addr: 0, generic_idx: 1 });
        assert_eq!(parsed.schemas[3].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        assert_eq!(parsed.schemas[4].data_type, NP_Schema_Type::Custom { type_idx: 0 });
        assert_eq!(parsed.schemas[4].id, Some(3));
        assert_eq!(parsed.schemas[4].generics, NP_Parsed_Generics::Types(vec![5, 6]));
        assert_eq!(parsed.name_index.get("anotherType"), Some(&NP_Schema_Index { data: 4, methods: None }));
        assert_eq!(parsed.id_index.get(3), Some(&NP_Schema_Index { data: 4, methods: None }));

        assert_eq!(parsed.schemas[5].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });
        assert_eq!(parsed.schemas[6].data_type, NP_Schema_Type::Int64 { default: Default::default(), max: None, min: None });

        assert_eq!(parsed.schemas[7].data_type, NP_Schema_Type::Custom { type_idx: 0 });
        assert_eq!(parsed.schemas[7].generics, NP_Parsed_Generics::Types(vec![8, 10]));
        assert_eq!(parsed.name_index.get("crazyType"), Some(&NP_Schema_Index { data: 7, methods: None }));
        assert_eq!(parsed.id_index.get(4), Some(&NP_Schema_Index { data: 7, methods: None }));

        assert_eq!(parsed.schemas[8].data_type, NP_Schema_Type::Vec { max_len: None });
        assert_eq!(parsed.schemas[8].generics, NP_Parsed_Generics::Types(vec![9]));

        assert_eq!(parsed.schemas[9].data_type, NP_Schema_Type::Uint32 { default: Default::default(), max: None, min: None });
        assert_eq!(parsed.schemas[10].data_type, NP_Schema_Type::Int64 { default: Default::default(), max: None, min: None });

        Ok(())
    }

    #[test]
    fn impl_test_1() -> Result<(), NP_Error> {

        let schema = r##"
            struct myType [ id: 2 ] {
                username: string,
                email: string
            }

            impl myType {
                get(id: uuid) -> Option<self>,
                set(self) -> Result<(), string>
            }
        "##;

        let parsed = NP_Schema::parse(schema)?;
        // assert_eq!(NP_Schema::from_bytes(&parsed.to_bytes()?)?, parsed);


        assert_eq!(parsed.schemas.len(), 13);
        assert_eq!(parsed.name_index.get("myType"), Some(&NP_Schema_Index { data: 0, methods: Some(3) }));
        assert_eq!(parsed.id_index.get(2), Some(&NP_Schema_Index { data: 0, methods: Some(3) }));
        assert_eq!(parsed.schemas[0].arguments.query("id", schema), Some(NP_Args::NUMBER("2")));

        if let NP_Schema_Type::Struct { children } = &parsed.schemas[0].data_type {
            assert_eq!(children.iter_keys().collect::<Vec<&String>>(), vec!["username", "email"]);
            for (key, value) in children.iter() {
                match key.as_str() {
                    "username" => assert_eq!(*value, 1),
                    "email" => assert_eq!(*value, 2),
                    _ => assert!(false)
                }
            }
        } else {
            assert!(false);
        }

        assert_eq!(parsed.schemas[0].id, Some(2));
        assert_eq!(parsed.schemas[0].name.unwrap().read(schema), "myType");


        assert_eq!(parsed.schemas[1].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });
        assert_eq!(parsed.schemas[2].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        let mut impl_hash: NP_HashMap<usize> = NP_HashMap::new();
        impl_hash.set("get", 4)?;
        impl_hash.set("set", 8)?;
        assert_eq!(parsed.schemas[3].data_type, NP_Schema_Type::Impl { children: impl_hash });

        if let NP_Schema_Type::Method { args, returns } = &parsed.schemas[4].data_type {
            assert_eq!(args.get("id"), Some(&5));
            assert_eq!(returns, &6);
        }

        assert_eq!(parsed.schemas[5].data_type, NP_Schema_Type::Uuid);
        assert_eq!(parsed.schemas[6].data_type, NP_Schema_Type::Option);
        assert_eq!(parsed.schemas[6].generics, NP_Parsed_Generics::Types(vec![7]));
        assert_eq!(parsed.schemas[7].data_type, NP_Schema_Type::Fn_Self { idx: 3 });

        if let NP_Schema_Type::Method { args, returns } = &parsed.schemas[8].data_type {
            assert_eq!(args.get("self"), Some(&9));
            assert_eq!(returns, &10);
        }

        assert_eq!(parsed.schemas[9].data_type, NP_Schema_Type::Fn_Self { idx: 3 });

        assert_eq!(parsed.schemas[10].data_type, NP_Schema_Type::Result);
        assert_eq!(parsed.schemas[10].generics, NP_Parsed_Generics::Types(vec![11, 12]));

        assert_eq!(parsed.schemas[11].data_type, NP_Schema_Type::Tuple { children: vec![] });
        assert_eq!(parsed.schemas[12].data_type, NP_Schema_Type::String { default: Default::default(), casing: Default::default(), max_len: None });

        Ok(())
    }

}