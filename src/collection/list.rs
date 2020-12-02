use crate::{buffer::LIST_MAX_SIZE, schema::NP_Schema_Addr};
use crate::{error::NP_Error, json_flex::{JSMAP, NP_JSON}, memory::{NP_Memory, NP_Size}, pointer::{NP_Value}, pointer::{NP_Cursor, NP_Cursor_Value, NP_Cursor_Parent}, schema::NP_Parsed_Schema, schema::{NP_Schema, NP_TypeKeys}};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::{vec::*};
use core::{hint::unreachable_unchecked};
use alloc::string::ToString;

/// List data type.
/// 
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_List<'list> { 
    cursor: NP_Cursor,
    list: NP_Cursor_Parent,
    list_of_addr: NP_Schema_Addr,
    current: Option<(usize, NP_Cursor)>,
    pub memory: &'list NP_Memory<'list>,
    real_only: bool
}

impl<'list> NP_List<'list> {

    /// Generate a new list iterator
    /// 
    #[inline(always)]
    pub fn new(mut cursor: NP_Cursor, memory: &'list NP_Memory<'list>, real_only: bool) -> Self {
        let value_addr = if cursor.buff_addr != 0 { memory.read_address(cursor.buff_addr) } else { 0 };
        cursor.value = cursor.value.update_value_address(value_addr);
        let addr_size = memory.addr_size_bytes();
        Self {
            cursor: cursor,
            list: NP_Cursor_Parent::List {
                head: memory.read_address(value_addr),
                tail: memory.read_address(value_addr + addr_size),
                addr: value_addr,
                schema_addr: cursor.schema_addr
            },
            list_of_addr: match memory.schema[cursor.schema_addr] {
                NP_Parsed_Schema::List { of, ..} => {
                    of
                },
                _ => { unsafe { unreachable_unchecked() } }
            },
            current: None,
            memory: memory,
            real_only
        }
    }

    /// Read or save a list into the buffer
    /// 
    #[inline(always)]
    pub fn read_list(buff_addr: usize, schema_addr: usize, memory: &NP_Memory<'list>, create: bool) -> Result<(NP_Cursor, usize, usize), NP_Error> {

        let mut cursor = NP_Cursor::new(buff_addr, schema_addr, &memory, NP_Cursor_Parent::None);
        let mut value_addr = cursor.value.get_value_address();
        let addr_size = memory.addr_size_bytes();
        
        if value_addr == 0 { // no list here
            if create { // please make one
                assert_ne!(cursor.buff_addr, 0); 
                value_addr = match memory.size { // stores HEAD & TAIL for list
                    NP_Size::U8 => {  memory.malloc_borrow(&[0u8; 2])? },
                    NP_Size::U16 => { memory.malloc_borrow(&[0u8; 4])? },
                    NP_Size::U32 => { memory.malloc_borrow(&[0u8; 8])? }
                };
                // update buffer
                memory.write_address(cursor.buff_addr, value_addr);
                // update cursor
                cursor.value = cursor.value.update_value_address(value_addr);
                Ok((cursor, 0, 0))
            } else { // no list and no need to make one, just pass empty data
                Ok((cursor, 0, 0))       
            }
        } else { // list found, read info from buffer
            Ok((cursor, memory.read_address(value_addr), memory.read_address(value_addr + addr_size)))
        }
    }

    /// Accepts a cursor that is currently on a list type and moves the cursor to a list item
    #[inline(always)]
    pub fn select_into(cursor: NP_Cursor, memory: &'list NP_Memory<'list>, create_path: bool, index: usize) -> Result<NP_Cursor, NP_Error> {

        let addr_size = memory.addr_size_bytes();

        let (mut head, mut tail, list_addr, list_schema_addr) = {
            let list_cursor = Self::read_list(cursor.buff_addr, cursor.schema_addr, &memory, create_path)?;
            (list_cursor.1, list_cursor.2, list_cursor.0.value.get_value_address(), list_cursor.0.schema_addr)
        };

        let list_of_schema_addr = match memory.schema[list_schema_addr] {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        // if head is zero there are no items in the list
        // if create_path is true we're going to make a new list item as the head/tail
        // otherwise we'll return a virtual cursor of the requested index
        if head == 0 {
            let mut virtual_cursor = NP_Cursor::new(0, list_of_schema_addr, memory, NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  });
            virtual_cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, index: index, next: 0 };

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // update list head and tail
                head = virtual_cursor.buff_addr;
                tail = virtual_cursor.buff_addr;
                memory.write_address(list_addr, head);
                memory.write_address(list_addr + addr_size, tail);

                // write index into new cursor
                if memory.size == NP_Size::U8 {
                    memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size] = index as u8;
                } else {
                    for (i, b) in (index as u16).to_be_bytes().iter().enumerate() {
                        memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size + i] = *b;
                    }
                }
                virtual_cursor.parent = NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  };
            }

            return Ok(virtual_cursor)
        }

        

        let head_cursor = NP_Cursor::new(head, list_of_schema_addr, &memory, NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  });

        let (_head_value, _head_next, head_index ) = match head_cursor.value {
            NP_Cursor_Value::ListItem { value_addr, next, index } => { (value_addr, next, index)},
            _ => unsafe { unreachable_unchecked() }
        };

        // head/first pointer matches requested index, return the head pointer
        if head_index == index {
            return Ok(head_cursor);
        }

        // requested index is in front of head
        if head_index > index  {

            let mut virtual_cursor = NP_Cursor::new(0, list_of_schema_addr, memory, NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  });
            
            virtual_cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, index: index, next: head };

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // write index into new cursor
                if memory.size == NP_Size::U8 {
                    memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size] = index as u8;
                } else {
                    for (i, b) in (index as u16).to_be_bytes().iter().enumerate() {
                        memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size + i] = *b;
                    }
                }
                
                // write next into new cursor
                memory.write_address(virtual_cursor.buff_addr + addr_size, head);

                // update list head
                head = virtual_cursor.buff_addr;
                memory.write_address(list_addr, head);
            }

            virtual_cursor.parent = NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  };

            return Ok(virtual_cursor)
        }
        
        let tail_cursor = NP_Cursor::new(tail, list_of_schema_addr, &memory, NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  });

        let (_tail_value, _tail_next, tail_index ) = match tail_cursor.value {
            NP_Cursor_Value::ListItem { value_addr, next, index } => { (value_addr, next, index)},
            _ => unsafe { unreachable_unchecked() }
        };

        // requesting index higher than tail index
        if tail_index < index {
            let mut virtual_cursor = NP_Cursor::new(0, list_of_schema_addr, memory, NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  });
            
            virtual_cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, index: index, next: 0 };

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // write index into new cursor
                if memory.size == NP_Size::U8 {
                    memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size] = index as u8;
                } else {
                    for (i, b) in (index as u16).to_be_bytes().iter().enumerate() {
                        memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size + i] = *b;
                    }
                }
                
                // write next into old tail
                memory.write_address(tail + addr_size, virtual_cursor.buff_addr);

                // update list tail
                tail = virtual_cursor.buff_addr;
                memory.write_address(list_addr + addr_size, tail);
            }

            virtual_cursor.parent = NP_Cursor_Parent::List { head: head, tail: tail, addr: list_addr, schema_addr: list_schema_addr  };

            return Ok(virtual_cursor)
        }

        // index is somewhere in the existing records, loop time!
        for (idx, item) in Self::new(cursor.clone(), &memory, false) {
            if idx == index {
                if item.buff_addr == 0 && create_path { // need to convert virtual cursor into real one
                    let saved = NP_List::commit_virtual_cursor(item.clone(), &memory)?;
                    return Ok(saved)
                } else {
                    return Ok(item.clone())
                }
            }
        }

        // should never reach this
        panic!()
    }

    /// Commit a virtual cursor into the buffer
    /// 
    pub fn commit_virtual_cursor<'commit>(mut cursor: NP_Cursor, memory: &'commit NP_Memory<'commit>) -> Result<NP_Cursor, NP_Error> {
        
        if cursor.buff_addr != 0 {
            return Ok(cursor)
        };

        let addr_size = memory.addr_size_bytes();

        cursor.buff_addr = memory.malloc_cursor(&cursor.value)?;

        let index = match cursor.value {
            NP_Cursor_Value::ListItem { index , .. } => { index },
            _ => { unsafe { unreachable_unchecked() }}
        };

        match cursor.parent {
            NP_Cursor_Parent::List { head: _ , tail: _ , schema_addr: _, addr } => {

                if let Some(prev) = cursor.prev_cursor { // update previous cursor to point to this one
                    memory.write_address(prev + addr_size, cursor.buff_addr);
                } else { // update head to point to this cursor
                    // head = cursor.buff_addr;
                    memory.write_address(addr, cursor.buff_addr);
                }

                let mut next_addr = 0;

                if let Some(next) = cursor.next_cursor { // update this pointer to point to next one
                    next_addr = next;
                    memory.write_address(cursor.buff_addr + addr_size, next);
                } else { // update tail
                    // tail = cursor.buff_addr;
                    memory.write_address(addr, cursor.buff_addr);
                }

                // update index
                match memory.size {
                    NP_Size::U8 => {
                        memory.write_bytes()[cursor.buff_addr + addr_size + addr_size] = index as u8;
                    },
                    _ => {
                        for (i, b) in (index as u16).to_be_bytes().iter().enumerate() {
                            memory.write_bytes()[cursor.buff_addr + addr_size + addr_size + i] = *b;
                        }
                    }
                }

                cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, next: next_addr, index: index };

                return Ok(cursor);
            },
            _ => { unsafe { unreachable_unchecked() }}
        }
    }


    /// Push a new value onto the back of the list
    pub fn push(list_cursor: NP_Cursor, memory: &'list NP_Memory<'list>, index: Option<u16>) -> Result<(usize, NP_Cursor), NP_Error> {

        let (list_cursor, head, mut tail) = Self::read_list(list_cursor.buff_addr, list_cursor.schema_addr, &memory, true)?;

        let list_value_addr = list_cursor.value.get_value_address();

        let addr_size = memory.addr_size_bytes();

        let list_of = match &memory.schema[list_cursor.schema_addr] {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                *of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        if head == 0 { // no values in list, return new cursor at index 0

            let new_cursor = Self::select_into(list_cursor.clone(), &memory, true, index.unwrap_or(0).into())?;
            Ok((0, new_cursor))

        } else { // get tail information and return virtual pointer behind tail
 
            let tail_cursor = NP_Cursor::new(tail, list_of, &memory, NP_Cursor_Parent::List {
                head, tail, addr: list_cursor.buff_addr, schema_addr: list_cursor.schema_addr
            });

            let tail_index = match tail_cursor.value {
                NP_Cursor_Value::ListItem { index, ..} => index,
                _ => unsafe { unreachable_unchecked() }
            };

            if (tail_index + 1) > LIST_MAX_SIZE {
                return Err(NP_Error::new("Error pushing list, out of space!"));
            }

            // auto generate new index from the tail pointer
            let mut new_index = tail_index + 1;

            // if we were given an index to assign to this pointer, make sure it's higher than the generated one
            if let Some(x ) = index {
                if x < new_index as u16 {
                    return Err(NP_Error::new(String::from("Requested index is lower than last item, can't push!")));
                } else {
                    new_index = x as usize;
                }
            }

            let new_cursor_addr = memory.malloc_cursor(&NP_Cursor_Value::ListItem { value_addr: 0, index: 0, next: 0})?;

            // update index in buffer
            match memory.size {
                NP_Size::U8 => {
                    memory.write_bytes()[new_cursor_addr + addr_size + addr_size] = new_index as u8;
                },
                _ => {
                    for (i, b) in (new_index as u16).to_be_bytes().iter().enumerate() {
                        memory.write_bytes()[new_cursor_addr + addr_size + addr_size + i] = *b;
                    }
                }
            }

            // update "NEXT" value of old tail pointer
            memory.write_address(tail + addr_size, new_cursor_addr);

            // update tail value to this new pointer
            tail = new_cursor_addr;
            memory.write_address(list_value_addr + addr_size, new_cursor_addr);

            return Ok((new_index, NP_Cursor::new(new_cursor_addr, list_of, &memory, NP_Cursor_Parent::List {
                head, tail, addr: list_cursor.buff_addr, schema_addr: list_cursor.schema_addr
            })));
        }
    }
}

impl<'value> NP_Value<'value> for NP_List<'value> {

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

    fn get_size(cursor: NP_Cursor, memory: &NP_Memory) -> Result<usize, NP_Error> {

        if cursor.value.get_value_address() == 0 {
            return Ok(0) 
        }

        // head + tail;,
        let base_size = match memory.size {
            NP_Size::U32 => 8usize,
            NP_Size::U16 => 4usize,
            NP_Size::U8 => 2usize
        };

        let mut acc_size = 0usize;
 
        for (_i, item) in NP_List::new(cursor.clone_lite(), memory, true) {
            acc_size += NP_Cursor::calc_size(item.clone(), memory).unwrap();
        }


        Ok(acc_size + base_size)
    }
    
    fn to_json(cursor: &NP_Cursor, memory: &NP_Memory) -> NP_JSON {

        if cursor.buff_addr == 0 { return NP_JSON::Null };

        let mut json_list = Vec::new();

        for (_i, item) in NP_List::new(cursor.clone_lite(), memory, false) {
            json_list.push(NP_Cursor::json_encode(&item, memory));      
        }

        NP_JSON::Array(json_list)
    }

    fn do_compact(from_cursor: &NP_Cursor, from_memory: &NP_Memory<'value>, to_cursor: NP_Cursor, to_memory: &NP_Memory<'value>) -> Result<NP_Cursor, NP_Error> where Self: 'value {

        if from_cursor.buff_addr == 0 || from_cursor.value.get_value_address() == 0 {
            return Ok(to_cursor);
        }

        for (index, old_item) in NP_List::new(from_cursor.clone(), from_memory, true) {
            if old_item.buff_addr != 0 && old_item.value.get_value_address() != 0 { // pointer has value
                let (_new_index, new_item) = NP_List::push(to_cursor.clone(), to_memory, Some(index as u16))?;
                NP_Cursor::compact(&old_item, from_memory, new_item, to_memory)?;
            } 
        }

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

impl<'it> Iterator for NP_List<'it> {
    
    type Item = (usize, NP_Cursor);

    fn next(&mut self) -> Option<Self::Item> {

        if let Some((mut current_index, mut current)) = self.current { // step pointer
            if let Some(next) = current.next_cursor {

                let mut next_cursor = NP_Cursor::new(next, current.schema_addr, &self.memory, current.parent.clone());

                let (next_index, next_next_addr) = match next_cursor.value {
                    NP_Cursor_Value::ListItem { index, next, ..} => { (index, next )},
                    _ => { unsafe { unreachable_unchecked() } }
                };

                if next_index == current_index + 1 || self.real_only { // next step into next_cursor
                    next_cursor.prev_cursor = Some(current.buff_addr);

                    if next_next_addr == 0 {
                        next_cursor.next_cursor = None;
                    } else {
                        next_cursor.next_cursor = Some(next_next_addr);
                    }
                    
                    current_index = next_index;
                    self.current = Some((current_index, next_cursor));

                    match self.current { Some(x) => Some(x), None => None }
                } else { // next step into virtual cursor
                    current.buff_addr = 0;
                    current_index += 1;
                    current.value = NP_Cursor_Value::ListItem { index: current_index, value_addr: 0, next: 0};
                    self.current = Some((current_index, current));

                    match self.current { Some(x) => Some(x), None => None }
                }


            } else { // nothing left in list
                None
            }

        } else { // make first pointer
            let mut first_pointer = NP_List::select_into(self.cursor.clone(), &self.memory, false, 0).unwrap();

            match first_pointer.value {
                NP_Cursor_Value::ListItem { next, ..} => {
                    if next != 0 {
                        first_pointer.next_cursor = Some(next);
                    }
                },
                _ => { unsafe { unreachable_unchecked() }}
            }

            self.current = Some((0, first_pointer));

            match self.current { Some(x) => Some(x), None => None }
        }
    }

    fn count(self) -> usize where Self: Sized {

        let list_addr = self.cursor.value.get_value_address();

        if self.cursor.buff_addr == 0 || list_addr == 0 {
            return 0;
        }

        let addr_size = self.memory.addr_size_bytes();
        let tail_addr = self.memory.read_address(list_addr + addr_size);

        if tail_addr == 0 {
            return 0;
        }

        let tail_cursor = NP_Cursor::new(tail_addr, self.list_of_addr, &self.memory, self.list.clone());

        match tail_cursor.value {
            NP_Cursor_Value::ListItem { index, ..} => {
                return index + 1;
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
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
    let mut buffer = factory.empty_buffer(None, None)?;
    buffer.set(&["10"], "hello, world")?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 28usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // values preserved through compaction
    let mut buffer = factory.empty_buffer(None, None)?;
    buffer.set(&["10"], "hello, world")?;
    buffer.set(&["12"], "hello, world2")?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["12"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 49usize);
    buffer.compact(None, None)?;
    assert_eq!(buffer.get::<&str>(&["10"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["12"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 49usize);

    Ok(())
}