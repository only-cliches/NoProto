use crate::pointer::NP_Scalar;
use crate::{pointer::{NP_List_Bytes}, schema::NP_Schema_Addr};
use crate::{error::NP_Error, json_flex::{JSMAP, NP_JSON}, memory::{NP_Memory}, pointer::{NP_Value}, pointer::{NP_Cursor}, schema::NP_Parsed_Schema, schema::{NP_Schema, NP_TypeKeys}};

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::{vec::*};
use core::{hint::unreachable_unchecked};
use alloc::string::ToString;

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
struct List_Item {
    index: usize,
    buff_addr: usize
}

/// List data type.
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_List {
    current: Option<List_Item>,
    previous: Option<List_Item>,
    index: usize,
    tail: Option<List_Item>,
    head: Option<List_Item>,
    only_real: bool,
    schema_of: usize,
    list_schema: usize,
}


impl NP_List {

    #[inline(always)]
    pub fn select(list_cursor: NP_Cursor, index: usize) -> Result<(usize, Option<NP_Cursor>), NP_Error> {
        panic!()
    }

    pub fn make_list<'make>(list_cursor_addr: &NP_Cursor, memory: &'make NP_Memory) -> Result<(), NP_Error> {

        let cursor = memory.get_parsed(list_cursor_addr);

        let list_addr = memory.malloc_borrow(&[0u8; 4])?; // head & tail
                
        cursor.value.set_addr_value(list_addr as u16);
        cursor.data = NP_Cursor_Data::List { list_addrs: [0; 255], bytes: unsafe { &mut *(memory.write_bytes().as_ptr().add(list_addr as usize) as *mut NP_List_Bytes) } };

        Ok(())
    }

    pub fn new_iter(list_cursor: &NP_Cursor, memory: &NP_Memory, only_real: bool) -> Self {

        let value = list_cursor.get_value(memory);

        let list_addr = value.get_addr_value() as usize;

        let schema_of = match memory.schema[list_cursor.schema_addr] {
            NP_Parsed_Schema::List { of, .. } => of,
            _ => unsafe { unreachable_unchecked() }
        };

        let index = 0usize;
        let mut tail_index = 0usize;

        if list_addr > 0 {

            let bytes = unsafe { &mut *(memory.write_bytes().as_ptr().add(list_addr) as *const NP_List_Bytes) };

            let tail_addr = bytes.get_tail() as usize;

            if tail_addr != 0 { 
            
                let tail_cursor = NP_Cursor::new(tail_addr, schema_of, list_cursor.schema_addr);
                let head_cursor = NP_Cursor::new(bytes.get_head() as usize, schema_of, list_cursor.schema_addr);
                
                return Self {
                    current: None,
                    previous: None,
                    head: Some(List_Item { index: head_cursor.get_value(memory).get_index() as usize, buff_addr: head_cursor.buff_addr}),
                    tail: Some(List_Item { index: tail_cursor.get_value(memory).get_index() as usize, buff_addr: tail_cursor.buff_addr}),
                    only_real,
                    index: 0,
                    schema_of,
                    list_schema: list_cursor.schema_addr
                }
            }           
        }

        Self {
            current: None,
            previous: None,
            head: None,
            tail: None,
            only_real,
            index: 0,
            schema_of,
            list_schema: list_cursor.schema_addr
        }
    }

    pub fn step_iter(&mut self, memory: &NP_Memory) -> Option<(usize, Option<NP_Cursor>)> {

        match self.head {
            Some(head) => {
                match self.tail {
                    Some(tail) => {
                        match self.current {
                            Some(current) => { // subsequent iterations
                                let current_cursor = NP_Cursor::new(current.buff_addr, self.schema_of, self.list_schema);
                                let value = current_cursor.get_value(memory);
                                if self.only_real {
                                    let next_addr = value.get_next_addr() as usize;
                                    if next_addr == 0 {
                                        None
                                    } else {
                                        let next_cursor = NP_Cursor::new(next_addr, self.schema_of, self.list_schema);
                                        let next_index = next_cursor.get_value(memory).get_index();
                                        self.index = next_index as usize;
                                        self.previous = self.current.clone();
                                        self.current = Some(List_Item { buff_addr: next_addr, index: next_index as usize});
                                        Some((next_index as usize, Some(next_cursor)))
                                    }
                                } else {

                                }
                                // panic!()
                            },
                            None => { // first iteration
                                self.current = Some(head);
                                if self.only_real || head.index == 0 {
                                    self.index = head.index;
                                    Some((head.index, Some(NP_Cursor::new(head.buff_addr, self.schema_of, self.list_schema))))
                                } else {
                                    self.index = 0;
                                    Some((0, None))
                                }
                            }
                        }
                    },
                    None => None
                }
            },
            None => None
        }
    }

    pub fn for_each<F>(cursor_addr: &NP_Cursor, memory: &NP_Memory, only_real: bool, callback: &mut F) where F: FnMut((usize, Option<NP_Cursor>)) {

        let mut list_iter = NP_List::new_iter(cursor_addr, memory, only_real);

        while let Some((index, item)) = NP_List::step_iter(&mut list_iter, memory) {
            callback((index, item))
        }

    }

    pub fn push<'push>(list_cursor_addr: &NP_Cursor, memory: &NP_Memory, index: Option<usize>) -> Result<Option<(u16, NP_Cursor)>, NP_Error> {

        let list_cursor = memory.get_parsed(&list_cursor_addr);

        match memory.schema[list_cursor.schema_addr] {
            NP_Parsed_Schema::List {  of, .. } => {

                let of_schema = &memory.schema[of];

                let mut new_index: usize = 0;

                match &mut list_cursor.data {
                    NP_Cursor_Data::List { list_addrs, bytes } => {

                        let new_item_addr = memory.malloc_borrow(&[0u8; 5])?;

                        let mut new_item_cursor = NP_Cursor { 
                            buff_addr: new_item_addr, 
                            schema_addr: of, 
                            data: NP_Cursor_Data::Empty,
                            temp_bytes: None,
                            value: NP_Cursor::parse_cursor_value(new_item_addr, list_cursor.buff_addr, list_cursor.schema_addr, &memory), 
                            parent_addr: list_cursor.buff_addr,
                            index: if let Some(idx) = index {
                                idx
                            } else {
                                0
                            }
                        };

                        if bytes.get_head() == 0 { // empty list
                            bytes.set_head(new_item_addr as u16);
                            bytes.set_tail(new_item_addr as u16);
                            new_item_cursor.value.set_index(new_item_cursor.index as u8);
                        } else { // list has items
                            let old_tail = memory.get_parsed(&NP_Cursor_Addr::Real(bytes.get_tail() as usize));
                            old_tail.value.set_next_addr(new_item_addr as u16);
                            new_index = if let Some(idx) = index {
                                idx as usize
                            } else {
                                old_tail.value.get_index() as usize + 1
                            };
                            new_item_cursor.value.set_index(new_index as u8);
                            new_item_cursor.index = new_index;
                            bytes.set_tail(new_item_addr as u16);
                        }

                        list_addrs[new_item_cursor.index] = new_item_addr;

                        memory.insert_parsed(new_item_addr, new_item_cursor);

                        return Ok(Some((new_index as u16, NP_Cursor_Addr::Real(new_item_addr))));
                    },
                    _ => unsafe { unreachable_unchecked() }
                }
            },
            _ => Ok(None)
        }
    }


    /// Commit a virtual cursor into the buffer
    /// 
    pub fn insert<'commit>(cursor_addr: &NP_Cursor, memory: &'commit NP_Memory, index: usize) -> Result<usize, NP_Error> {

        let cursor = memory.get_parsed(&cursor_addr);

        match &mut cursor.data {
            NP_Cursor_Data::List { list_addrs, bytes } => {
                match memory.schema[cursor.schema_addr] {
                    NP_Parsed_Schema::List { of, ..  } => {

                        if bytes.get_head() == 0 { // empty list

                            let new_item_addr = memory.malloc_borrow(&[0u8; 5])?;

                            bytes.set_head(new_item_addr as u16);
                            bytes.set_tail(new_item_addr as u16);

                            let new_item_cursor = NP_Cursor { 
                                buff_addr: new_item_addr, 
                                schema_addr: of, 
                                data: NP_Cursor_Data::Empty,
                                temp_bytes: None,
                                value: NP_Cursor::parse_cursor_value(new_item_addr, cursor.buff_addr, cursor.schema_addr, memory), 
                                parent_addr: cursor.buff_addr,
                                index
                            };

                            new_item_cursor.value.set_index(index as u8);

                            memory.insert_parsed(new_item_addr, new_item_cursor);

                            list_addrs[index] = new_item_addr;

                            return Ok(new_item_addr);
                        }

                        let new_item_addr = memory.malloc_borrow(&[0u8; 5])?;
                        
                        let mut new_item_cursor = NP_Cursor { 
                            buff_addr: new_item_addr, 
                            schema_addr: of, 
                            data: NP_Cursor_Data::Empty,
                            temp_bytes: None,
                            value: NP_Cursor::parse_cursor_value(new_item_addr, cursor.buff_addr, cursor.schema_addr, memory), 
                            parent_addr: cursor.buff_addr,
                            index
                        };

                        // find previous list item
                        let (head_index, head_addr) = {
                            let head_cursor = memory.get_parsed(&NP_Cursor_Addr::Real(bytes.get_head() as usize));
                            (head_cursor.value.get_index() as usize, head_cursor.buff_addr)
                        };

                        let (tail_index, tail_addr) = {
                            let tail_index = memory.get_parsed(&NP_Cursor_Addr::Real(bytes.get_tail() as usize));
                            (tail_index.value.get_index() as usize, tail_index.buff_addr)
                        };

                        if head_index > index { // we have a new head
                            new_item_cursor.value.set_next_addr(head_addr as u16);
                            bytes.set_head(new_item_addr as u16);
                        } else if tail_index < index { // we have a new tail
                            let old_tail = memory.get_parsed(&NP_Cursor_Addr::Real(tail_addr));
                            old_tail.value.set_next_addr(new_item_addr as u16);
                            bytes.set_tail(new_item_addr as u16);
                        } else { // somehwere in the middle
                            let mut walking_index = index;
                            while list_addrs[walking_index] == 0 {
                                walking_index -= 1;
                            }

                            let prev_addr = memory.get_parsed(&NP_Cursor_Addr::Real(list_addrs[walking_index]));
                            new_item_cursor.value.set_next_addr(prev_addr.value.get_next_addr());
                            prev_addr.value.set_next_addr(new_item_addr as u16);
                        }

                        new_item_cursor.value.set_index(index as u8);
                        memory.insert_parsed(new_item_addr, new_item_cursor);

                        list_addrs[index] = new_item_addr;

                        Ok(new_item_addr)
                    },
                    _ => unsafe { unreachable_unchecked() }
                }
            },
            _ => unsafe { unreachable_unchecked() }
        }
    }


}

impl<'value> NP_Value<'value> for NP_List {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("list", NP_TypeKeys::List) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("list", NP_TypeKeys::List) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));


        let list_of = match &schema[address] {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                *of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("of".to_owned(), NP_Schema::_type_to_json(schema, list_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn get_size(cursor: NP_Cursor_Addr, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {

        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 {
            return Ok(0) 
        }

        // head + tail;,
        let base_size = 4usize;

        let mut acc_size = 0usize;

        Self::for_each(&cursor, memory, true, &mut |(_i, item)| {
            acc_size += NP_Cursor::calc_size(item.clone(), memory).unwrap();
        });
 

        Ok(acc_size + base_size)
    }
    
    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_list = Vec::new();

        Self::for_each(&cursor, memory, true, &mut |(_i, item)| {
            json_list.push(NP_Cursor::json_encode(item.clone(), memory));     
        });

        NP_JSON::Array(json_list)
    }

    fn do_compact(from_cursor: &NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor: NP_Cursor_Addr, to_memory: &NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: Sized {

        let from_c = from_memory.get_parsed(from_cursor);

        if from_c.value.get_addr_value() == 0 {
            return Ok(to_cursor);
        }

        Self::make_list(&to_cursor, to_memory)?;

        let to_c = to_memory.get_parsed(&to_cursor);

        Self::for_each(from_cursor, from_memory, true, &mut |(index, item)| {
            let old_item = from_memory.get_parsed(&item);
            if old_item.buff_addr != 0 && old_item.value.get_addr_value() != 0 { // pointer has value
                let (_new_index, new_item) = NP_List::push(&to_cursor, to_memory, Some(index)).unwrap().unwrap();
                NP_Cursor::compact(&item, from_memory, new_item, to_memory).unwrap();
            }    
        });

        Ok(to_cursor)
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::List as u8);

        let list_schema_addr = schema.len();
        schema.push(NP_Parsed_Schema::List {
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

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {

        let list_schema_addr = schema.len();
        schema.push(NP_Parsed_Schema::List {
            i: NP_TypeKeys::List,
            sortable: false,
            of: list_schema_addr + 1
        });
        
        let (_sortable, schema) = NP_Schema::from_bytes(schema, address + 1, bytes);

        (false, schema)
    }
}



#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"list\",\"of\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"list\",\"of\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;

    // compaction removes values no longer in buffer
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["10"], "hello, world")?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 25usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 2usize);

    // values preserved through compaction
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["10"], "hello, world")?;
    buffer.set(&["12"], "hello, world2")?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["12"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 45usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["12"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 45usize);

    Ok(())
}

#[test]
fn parseing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"list\",\"of\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;

    // compaction removes values no longer in buffer
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["9"], "hello")?;
    buffer.set(&["10"], "world")?;
    let new_buffer = factory.open_buffer(buffer.close())?;
    assert_eq!(new_buffer.get::<&str>(&["9"])?.unwrap(), "hello");
    assert_eq!(new_buffer.get::<&str>(&["10"])?.unwrap(), "world");

    Ok(())
}