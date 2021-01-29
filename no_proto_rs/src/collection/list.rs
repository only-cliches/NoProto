use alloc::string::String;
use crate::{idl::{JS_AST, JS_Schema}, schema::NP_Value_Kind, utils::opt_err};
use crate::{error::NP_Error, json_flex::{JSMAP, NP_JSON}, memory::{NP_Memory}, pointer::{NP_Value}, pointer::{NP_Cursor}, schema::NP_Parsed_Schema, schema::{NP_Schema, NP_TypeKeys}};

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::{vec::*};
use alloc::string::ToString;


#[repr(C)]
#[derive(Debug)]
#[doc(hidden)]
#[allow(missing_docs)]
pub struct NP_List_Bytes {
    head: [u8; 2],
    tail: [u8; 2]
}

#[allow(missing_docs)]
impl NP_List_Bytes {
    #[inline(always)]
    pub fn set_head(&mut self, head: u16) {
        self.head = head.to_be_bytes();
    }
    #[inline(always)]
    pub fn get_head(&self) -> u16 {
        u16::from_be_bytes(self.head)
    }
    #[inline(always)]
    pub fn set_tail(&mut self, tail: u16) {
        self.tail = tail.to_be_bytes();
    }
    #[inline(always)]
    pub fn get_tail(&self) -> u16 {
        u16::from_be_bytes(self.tail)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct List_Item {
    index: usize,
    buff_addr: usize
}

/// List data type.
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_List {
    next: Option<List_Item>,
    current: Option<List_Item>,
    index: usize,
    count: usize,
    tail: Option<List_Item>,
    head: Option<List_Item>,
    only_real: bool,
    schema_of: usize,
    list: NP_Cursor
}


#[allow(missing_docs)]
impl NP_List {

    #[inline(always)]
    pub fn select<M: NP_Memory>(list_cursor: NP_Cursor, index: usize, make_path: bool, schema_query: bool, memory: &M) -> Result<Option<(usize, Option<NP_Cursor>)>, NP_Error> {
        let list_value = || { list_cursor.get_value(memory) };

        if index > 255 { return Ok(None) }

        let schema_of = match memory.get_schema(list_cursor.schema_addr) {
            NP_Parsed_Schema::List { of, .. } => *of,
            _ => 0
        };

        if schema_query {
            return Ok(Some((index, Some(NP_Cursor::new(0, schema_of, list_cursor.schema_addr)))));
        }

        // if no list here, make one please
        if list_value().get_addr_value() == 0 {
            if make_path {
                Self::make_list(&list_cursor, memory)?;
            } else {
                return Ok(Some((index, None)))
            }
        }

        let list_data = || {
            Self::get_list(list_value().get_addr_value() as usize, memory)
        }; 

        // empty list
        if list_data().get_head() == 0 {
            let new_cursor_addr = memory.malloc_borrow(&[0u8; 5])?; // malloc list item
            let new_cursor = NP_Cursor::new(new_cursor_addr, schema_of, list_cursor.schema_addr);
            let new_cursor_value = new_cursor.get_value(memory);
            new_cursor_value.set_index(index as u8);
            list_data().set_head(new_cursor_addr as u16);
            list_data().set_tail(new_cursor_addr as u16);
            return Ok(Some((index, Some(new_cursor))))
        }

        
        let head = NP_Cursor::new(list_data().get_head() as usize, schema_of, list_cursor.schema_addr);

        let head_index = head.get_value(memory).get_index() as usize;

        if head_index > index { // index is in front of head, replace head
            let new_cursor_addr = memory.malloc_borrow(&[0u8; 5])?; // malloc list item
            let new_cursor = NP_Cursor::new(new_cursor_addr, schema_of, list_cursor.schema_addr);
            let new_cursor_value = new_cursor.get_value(memory);
            new_cursor_value.set_index(index as u8);
            new_cursor_value.set_next_addr(head.buff_addr as u16);
            list_data().set_head(new_cursor_addr as u16);
            return Ok(Some((index, Some(new_cursor))))
        } else if head_index == index { // index is equal to head
            return Ok(Some((index, Some(head))))
        }

        // is cursor in behind of or equal to tail
        let tail = NP_Cursor::new(list_data().get_tail() as usize, schema_of, list_cursor.schema_addr);

        let tail_value = || { tail.get_value(memory) };
        let tail_index = tail_value().get_index() as usize;

        if tail_index < index { // index is behind tail
            let new_cursor_addr = memory.malloc_borrow(&[0u8; 5])?; // malloc list item
            let new_cursor = NP_Cursor::new(new_cursor_addr, schema_of, list_cursor.schema_addr);
            let new_cursor_value = new_cursor.get_value(memory);
            new_cursor_value.set_index(index as u8);
            tail_value().set_next_addr(new_cursor_addr as u16);
            list_data().set_tail(new_cursor_addr as u16);
            return Ok(Some((index, Some(new_cursor))))
        } else if tail_index == index { // index is equal to head
            return Ok(Some((index, Some(tail))))
        }

        // the index is somewhere in the list
        let mut list_iter = Self::new_iter(&list_cursor, memory, false, head_index as usize);

        while let Some((idx, item)) = Self::step_iter(&mut list_iter, memory) {
            if index == idx {
                if let Some(found_cursor) = item { // found cursor here
                    return Ok(Some((index, Some(found_cursor))))
                } else { // found index but no cursor
                    return Ok(Some((index, Some(list_iter.make_item_in_loop(memory)?))))
                }
            }
        }

        // should never reach here
        Err(NP_Error::Unreachable)

    }

    #[inline(always)]
    pub fn make_item_in_loop<M: NP_Memory>(self, memory: &M) -> Result<NP_Cursor, NP_Error> {

        let list_data = || { Self::get_list(self.list.get_value(memory).get_addr_value() as usize, memory) };

        let new_cursor_addr = memory.malloc_borrow(&[0u8; 5])?; // malloc list item
        let new_cursor = NP_Cursor::new(new_cursor_addr, self.schema_of, self.list.schema_addr);
        let new_cursor_value = new_cursor.get_value(memory);
        new_cursor_value.set_index(self.index as u8 - 1);


        if let Some(current) = self.current {

            // set NEXT of CURRENT cursor to the new cursor
            let curr_cursor = NP_Cursor::new(current.buff_addr, self.schema_of, self.list.schema_addr);
            let prev_cursor_value = curr_cursor.get_value(memory);
            prev_cursor_value.set_next_addr(new_cursor_addr as u16);

            if let Some(next) = self.next {
                new_cursor_value.set_next_addr(next.buff_addr as u16);
            } else { // replace tail
                list_data().set_tail(new_cursor_addr as u16);
            }

            Ok(new_cursor)
        } else {
            Err(NP_Error::Unreachable)
        }
    }

    #[inline(always)]
    pub fn make_list<'make, M: NP_Memory>(list_cursor: &NP_Cursor, memory: &'make M) -> Result<(), NP_Error> {
        let list_addr = memory.malloc_borrow(&[0u8; 4])?; // head & tail
        let value = list_cursor.get_value(memory);
        value.set_addr_value(list_addr as u16);
        Ok(())
    }

    #[inline(always)]
    pub fn get_list<'list, M: NP_Memory>(list_cursor_value_addr: usize, memory: &'list M) -> &'list mut NP_List_Bytes {
        if list_cursor_value_addr > memory.read_bytes().len() { // attack
            unsafe { &mut *(memory.write_bytes().as_ptr() as *mut NP_List_Bytes) }
        } else { // normal operation
            unsafe { &mut *(memory.write_bytes().as_ptr().add(list_cursor_value_addr as usize) as *mut NP_List_Bytes) }
        }
    }

    #[inline(always)]
    pub fn new_iter<M: NP_Memory>(list_cursor: &NP_Cursor, memory: &M, only_real: bool, starting_index: usize) -> Self {

        let value = list_cursor.get_value(memory);

        let list_addr = value.get_addr_value() as usize;

        let schema_of = match memory.get_schema(list_cursor.schema_addr) {
            NP_Parsed_Schema::List { of, .. } => *of,
            _ => 0
        };

        let memory_bytes = memory.write_bytes();

        if list_addr > 0 && list_addr < (memory_bytes.len() + 4) {

            let bytes = unsafe { &mut *(memory_bytes.as_ptr().add(list_addr) as *mut NP_List_Bytes) };

            let tail_addr = bytes.get_tail() as usize;

            if tail_addr != 0 { 
            
                let tail_cursor = NP_Cursor::new(tail_addr, schema_of, list_cursor.schema_addr);
                let head_cursor = NP_Cursor::new(bytes.get_head() as usize, schema_of, list_cursor.schema_addr);
                
                return Self {
                    current: None,
                    count: 0,
                    next: Some(List_Item { index: head_cursor.get_value(memory).get_index() as usize, buff_addr: head_cursor.buff_addr}),
                    head: Some(List_Item { index: head_cursor.get_value(memory).get_index() as usize, buff_addr: head_cursor.buff_addr}),
                    tail: Some(List_Item { index: tail_cursor.get_value(memory).get_index() as usize, buff_addr: tail_cursor.buff_addr}),
                    only_real,
                    index: starting_index,
                    schema_of,
                    list: list_cursor.clone(),
                }
            }           
        }

        Self {
            current: None,
            head: None,
            tail: None,
            count: 0,
            only_real,
            index: starting_index,
            schema_of,
            list: list_cursor.clone(),
            next: None,
        }
    }

    #[inline(always)]
    pub fn step_iter<M: NP_Memory>(&mut self, memory: &M) -> Option<(usize, Option<NP_Cursor>)> {

        if self.count > 255 {
            return None;
        }

        self.count += 1;

        match self.next {
            Some(next) => {

                if self.only_real {
                    self.current = self.next;
                    let this_cursor = NP_Cursor::new(next.buff_addr, self.schema_of, self.list.schema_addr);
                    let this_value = this_cursor.get_value(memory);
                    let next_addr = this_value.get_next_addr() as usize;
                    self.index = this_value.get_index() as usize;

                    if next_addr != 0 {
                        let next_cursor = NP_Cursor::new(next_addr, self.schema_of, self.list.schema_addr);
                        let next_index = next_cursor.get_value(memory).get_index() as usize;
                        self.next = Some(List_Item { index: next_index, buff_addr: next_addr });
                    } else {
                        self.next = None;
                    }
                    Some((self.index, Some(this_cursor)))
                } else {

                    if next.index > self.index {
                        self.index += 1;
                        Some((self.index - 1, None))
                    } else if next.index == self.index {
                        self.current = self.next;
                        let this_cursor = NP_Cursor::new(next.buff_addr, self.schema_of, self.list.schema_addr);
                        let this_value = this_cursor.get_value(memory);

                        let next_addr = this_value.get_next_addr() as usize;
                        self.index += 1;
    
                        if next_addr != 0 {
                            let next_cursor = NP_Cursor::new(next_addr, self.schema_of, self.list.schema_addr);
                            let next_index = next_cursor.get_value(memory).get_index() as usize;
                            self.next = Some(List_Item { index: next_index, buff_addr: next_addr });
                        } else {
                            self.next = None;
                        }

                        Some((self.index - 1, Some(this_cursor)))
                    } else {
                        None
                    }
                }
            },
            None => None
        }
    }

    #[inline(always)]
    pub fn push<'push, M: NP_Memory>(list_cursor: &NP_Cursor, memory: &M, index: Option<usize>) -> Result<Option<(u16, NP_Cursor)>, NP_Error> {

        let list_value = || {list_cursor.get_value(memory)};

        if list_value().get_addr_value() == 0 {
            Self::make_list(&list_cursor, memory)?;
        }

        match memory.get_schema(list_cursor.schema_addr) {
            NP_Parsed_Schema::List {  of, .. } => {

                let mut new_index: usize = index.unwrap_or(0);

                let new_item_addr = memory.malloc_borrow(&[0u8; 5])?; // list item

                let list_data = || {Self::get_list(list_value().get_addr_value() as usize, memory)};

                let new_cursor = NP_Cursor::new(new_item_addr, *of, list_cursor.schema_addr);
                let new_cursor_value = || {new_cursor.get_value(memory)};
                

                if list_data().get_head() == 0 { // empty list
                    list_data().set_head(new_item_addr as u16);
                    list_data().set_tail(new_item_addr as u16);
                    if new_index > 255 {
                        return Err(NP_Error::new("Index cannot be greater than 255!"))
                    }
                    new_cursor_value().set_index(new_index as u8)
                } else { // list has items
                    let old_tail = NP_Cursor::new(list_data().get_tail() as usize, *of, list_cursor.schema_addr);
                    let old_tail_value = || {old_tail.get_value(memory)};
                    old_tail_value().set_next_addr(new_item_addr as u16);
                    new_index = if let Some(idx) = index {
                        idx as usize
                    } else {
                        (old_tail_value().get_index() + 1) as usize
                    };
                    if new_index > 255 {
                        return Err(NP_Error::new("Index cannot be greater than 255!"))
                    }
                    new_cursor_value().set_index(new_index as u8);
                    list_data().set_tail(new_item_addr as u16);
                }


                return Ok(Some((new_index as u16, new_cursor)));
             
            },
            _ => Ok(None)
        }
    }
}

impl<'value> NP_Value<'value> for NP_List {

    fn to_json<M: NP_Memory>(depth:usize, cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {
        let c_value = || { cursor.get_value(memory) };

        if c_value().get_addr_value() == 0 {
            return NP_JSON::Null
        }

        let mut json_list = Vec::new();

        let mut list_iter = NP_List::new_iter(&cursor, memory, false, 0);

        while let Some((_index, item)) = NP_List::step_iter(&mut list_iter, memory) {
             if let Some(item_cursor) = &item {
                json_list.push(NP_Cursor::json_encode(depth + 1, item_cursor, memory));   
            } else {
                json_list.push(NP_JSON::Null);   
            }    
        }

        NP_JSON::Array(json_list)
    }

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("list", NP_TypeKeys::List) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("list", NP_TypeKeys::List) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));


        let list_of = match &schema[address] {
            NP_Parsed_Schema::List { of, .. } => { *of },
            _ => 0
        };

        schema_json.insert("of".to_owned(), NP_Schema::_type_to_json(schema, list_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_from_json<'set, M: NP_Memory>(depth: usize, apply_null: bool, cursor: NP_Cursor, memory: &'set M, value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized {

        match &**value {
            NP_JSON::Array(list) => {
                for (idx, list_item) in list.iter().enumerate() {
                    match NP_List::select(cursor, idx, true, false, memory)? {
                        Some(x) => {
                            match x.1 {
                                Some(list_value) => {
                                    NP_Cursor::set_from_json(depth + 1, apply_null, list_value, memory, &Box::new(list_item.clone()))?;
                                },
                                None => { }
                            }
                        },
                        None => { 
                            return Err(NP_Error::new("Failed to find field value!"))
                        }
                    }
                }
            },
            _ => { }
        }
        

        Ok(())
    }

    fn get_size<M: NP_Memory>(depth:usize, cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        let c_value = || { cursor.get_value(memory) };

        if c_value().get_addr_value() == 0 {
            return Ok(0) 
        }

        // head + tail
        let base_size = 4usize;

        let mut acc_size = 0usize;

        let mut list_iter = Self::new_iter(&cursor, memory, true, 0);

        while let Some((_index, item)) = Self::step_iter(&mut list_iter, memory) {
            if let Some(item_cursor) = &item {
                acc_size += NP_Cursor::calc_size(depth + 1, item_cursor, memory)?;
            }
        }

        Ok(acc_size + base_size)
    }
    


    fn do_compact<M: NP_Memory, M2: NP_Memory>(depth:usize, from_cursor: NP_Cursor, from_memory: &'value M, to_cursor: NP_Cursor, to_memory: &'value M2) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        let from_value = from_cursor.get_value(from_memory);

        if from_value.get_addr_value() == 0 {
            return Ok(to_cursor) 
        }

        Self::make_list(&to_cursor, to_memory)?;

        let mut list_iter = Self::new_iter(&from_cursor, from_memory, true, 0);

        while let Some((index, item)) = Self::step_iter(&mut list_iter, from_memory) {
            if let Some(old_item) = &item {
                let (_new_index, new_item) = opt_err(NP_List::push(&to_cursor, to_memory, Some(index))?)?;
                NP_Cursor::compact(depth + 1, old_item.clone(), from_memory, new_item, to_memory)?;
            }       
        }

        Ok(to_cursor)
    }

    fn schema_to_idl(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<String, NP_Error> {
        match &schema[address] {
            NP_Parsed_Schema::List { of, .. } => {
                let mut result = String::from("list({of: ");
                result.push_str(NP_Schema::_type_to_idl(&schema, *of)?.as_str());
                result.push_str("})");
                Ok(result)
            },
            _ => { Err(NP_Error::Unreachable) }
        }
    }

    fn from_idl_to_schema(mut schema: Vec<NP_Parsed_Schema>, _name: &str, idl: &JS_Schema, args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::List as u8);

        let list_schema_addr = schema.len();
        schema.push(NP_Parsed_Schema::List {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::List,
            of: list_schema_addr + 1,
            sortable: false
        });

        let mut of_jst: Option<&JS_AST> = None;

        if args.len() > 0 {
            match &args[0] {
                JS_AST::object { properties } => {
                    for (key, value) in properties {
                        if idl.get_str(key).trim() == "of" {
                            of_jst = Some(value);
                        }
                    }
                },
                _ => { }
            }
        };

        if let Some(x) = of_jst {
            // let of_addr = schema.len();
            let (_sortable, child_bytes, schema) = NP_Schema::from_idl(schema, idl, x)?;
            
            schema_bytes.extend(child_bytes);

            Ok((false, schema_bytes, schema))
        } else {
            Err(NP_Error::new("lists require an 'of' property!"))
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::List as u8);

        let list_schema_addr = schema.len();
        schema.push(NP_Parsed_Schema::List {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::List,
            of: list_schema_addr + 1,
            sortable: false
        });

        match json_schema["of"] {
            NP_JSON::Null => {
                return Err(NP_Error::new("Lists require an 'of' property that is a schema type!"))
            },
            _ => { }
        }

        // let of_addr = schema.len();
        let (_sortable, child_bytes, schema) = NP_Schema::from_json(schema, &Box::new(json_schema["of"].clone()))?;
        
        schema_bytes.extend(child_bytes);

        return Ok((false, schema_bytes, schema))
    }

    fn default_value(_depth: usize, _addr: usize, _schema: &Vec<NP_Parsed_Schema>) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {

        let list_schema_addr = schema.len();
        schema.push(NP_Parsed_Schema::List {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::List,
            sortable: false,
            of: list_schema_addr + 1
        });
        
        let (_sortable, schema) = NP_Schema::from_bytes(schema, address + 1, bytes);

        (false, schema)
    }
}


#[test]
fn schema_parsing_works_idl() -> Result<(), NP_Error> {
    let schema = r#"list({of: string()})"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl()?);
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl()?);
    Ok(())
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"list","of":{"type":"string"}}"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"list","of":{"type":"string"}}"#;
    let factory = crate::NP_Factory::new_json(schema)?;

    // compaction removes values no longer in buffer
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["10"], "hello, world")?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 27usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // values preserved through compaction
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["10"], "hello, world")?;
    buffer.set(&["12"], "hello, world2")?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["12"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 47usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["12"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 47usize);

    buffer.set_with_json(&[], r#"{"value": ["light", "this", "candle"]}"#)?;
    assert_eq!(buffer.get::<&str>(&["0"])?, Some("light"));
    assert_eq!(buffer.get::<&str>(&["1"])?, Some("this"));
    assert_eq!(buffer.get::<&str>(&["2"])?, Some("candle"));

    Ok(())
}

#[test]
fn parseing_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"list","of":{"type":"string"}}"#;
    let factory = crate::NP_Factory::new_json(schema)?;

    // compaction removes values no longer in buffer
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["9"], "hello")?;
    buffer.set(&["10"], "world")?;
    let new_buffer = factory.open_buffer(buffer.close());
    assert_eq!(new_buffer.get::<&str>(&["9"])?.unwrap(), "hello");
    assert_eq!(new_buffer.get::<&str>(&["10"])?.unwrap(), "world");

    Ok(())
}