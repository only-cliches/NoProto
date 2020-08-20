use crate::{memory::{NP_Size, NP_Memory}, pointer::{NP_Value, NP_Ptr, NP_PtrKinds, any::NP_Any, NP_Lite_Ptr}, error::NP_Error, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, json_flex::NP_JSON};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::{rc::Rc, vec::*};
/// List data type [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug)]
pub struct NP_List<T> {
    address: u32, // pointer location
    head: u32,
    tail: u32,
    memory: Option<Rc<NP_Memory>>,
    of: Option<Rc<NP_Schema>>,
    _value: T
}

impl<T: NP_Value + Default> NP_List<T> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, tail:u32,  memory: Rc<NP_Memory>, of: Rc<NP_Schema>) -> Result<Self, NP_Error> {

        // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
        if T::type_idx().0 != NP_TypeKeys::Any as i64 && of.type_data.0 != NP_TypeKeys::Any as i64  {

            // not using ANY casting, check type
            if of.type_data.0 != T::type_idx().0 {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str(T::type_idx().1.as_str());
                err.push_str(") to schema of type (");
                err.push_str(of.type_data.1.as_str());
                err.push_str(")");
                return Err(NP_Error::new(err));
            }
        }

        Ok(NP_List::<T> {
            address,
            head,
            tail,
            memory: Some(memory),
            of: Some(of),
            _value: T::default()
        })
    }

    /// Convert the list data type into an iterator
    pub fn it(self) -> NP_List_Iterator<T> {
        NP_List_Iterator::new(self.address, self.head, self.tail, self.memory.unwrap(), self.of.unwrap())
    }

    /// Select a value from the list.  If the value doesn't exist you'll get an empty pointer back.
    pub fn select(&mut self, index: u16) -> core::result::Result<NP_Ptr<T>, NP_Error> {

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let list_of = match &self.of {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        if self.head == 0 { // no values, create one

            let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::ListItem { addr: 0, i: 0, next: 0 }); // List item pointer

            // set index in pointer
            let index_bytes = index.to_be_bytes();

            match memory.size {
                NP_Size::U16 => {
                    for x in 0..index_bytes.len() {
                        ptr_bytes[x + 4] = index_bytes[x];
                    }
                },
                NP_Size::U32 => {
                    for x in 0..index_bytes.len() {
                        ptr_bytes[x + 8] = index_bytes[x];
                    }
                }
            };

            let addr = memory.malloc(ptr_bytes.to_vec())?;

            // update head to point to newly created ListItem pointer
            self.set_head(addr);
            self.set_tail(addr);
            
            // provide 
            return Ok(NP_Ptr::_new_list_item_ptr(self.head, list_of, memory));

        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0;

            let mut do_continue = true;

            while do_continue {

                let ptr_index: u16;

                let index_bytes: [u8; 2];

                let offset = match memory.size {
                    NP_Size::U32 => 8,
                    NP_Size::U16 => 4
                };

                match memory.get_2_bytes(curr_addr + offset) {
                    Some(x) => {
                        index_bytes = *x;
                    },
                    None => {
                        return Err(NP_Error::new("Out of range request"));
                    }
                }

                ptr_index = u16::from_be_bytes(index_bytes);

                // found our value!
                if ptr_index == index {
                    return Ok(NP_Ptr::_new_list_item_ptr(curr_addr as u32, list_of, memory));
                }

                // we've found an existing value above the requested index
                // insert a new pointer in before the current one in the loop
                if ptr_index > index {

                    let new_addr = {
            
                        let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::ListItem { addr: 0, i: 0, next: 0 }); // List item pointer

                        match &memory.size {
                            NP_Size::U16 => {
                               // set "next" value of this new pointer to current pointer in the loop
                               let curr_addr_bytes = (curr_addr as u16).to_be_bytes();
                               for x in 0..curr_addr_bytes.len() {
                                   ptr_bytes[2 + x] = curr_addr_bytes[x]; 
                               }

                               // set index of the new pointer
                               let index_bytes = index.to_be_bytes();
                               for x in 0..index_bytes.len() {
                                   ptr_bytes[4 + x] = index_bytes[x]; 
                               }
                            },
                            NP_Size::U32 => {
                                // set "next" value of this new pointer to current pointer in the loop
                                let curr_addr_bytes = (curr_addr as u32).to_be_bytes();
                                for x in 0..curr_addr_bytes.len() {
                                    ptr_bytes[4 + x] = curr_addr_bytes[x]; 
                                }

                                // set index of the new pointer
                                let index_bytes = index.to_be_bytes();
                                for x in 0..index_bytes.len() {
                                    ptr_bytes[8 + x] = index_bytes[x]; 
                                }
                            }
                        };

                        memory.malloc(ptr_bytes.to_vec())?
                    };

                    if curr_addr == self.head as usize {
                        // update head to new pointer
                        self.set_head(new_addr);
                    } else {
                        // update "next" value of previous pointer to the one we just made

                        let memory_write = memory.write_bytes();

                        match &memory.size {
                            NP_Size::U16 => {
                                let new_addr_bytes = (new_addr as u16).to_be_bytes();

                                for x in 0..new_addr_bytes.len() {
                                    memory_write[prev_addr + 2 + x] = new_addr_bytes[x];
                                }
                            },
                            NP_Size::U32 => {
                                let new_addr_bytes = new_addr.to_be_bytes();

                                for x in 0..new_addr_bytes.len() {
                                    memory_write[prev_addr + 4 + x] = new_addr_bytes[x];
                                }
                            }
                        };
                    }

                    return Ok(NP_Ptr::_new_list_item_ptr(new_addr as u32, list_of, memory));
                } else {
                    // not found yet, get next address

                    let next_ptr = match &memory.size {
                        NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(curr_addr + 2).unwrap_or(&[0; 2])) as usize,
                        NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4])) as usize
                    };
                    // let next_ptr = u32::from_be_bytes(next_bytes) as usize;
                    if next_ptr == 0 { // out of values to check
                        do_continue = false;
                    } else {
                        // store old value for next loop
                        prev_addr = curr_addr;

                        // set next pointer for next loop
                        curr_addr = next_ptr;
                    }
                }
            }

            
            // ran out of pointers to check, make one!
            let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::ListItem { addr: 0, i: 0, next: 0 }); // List item pointer

            // get index bytes
            let column_index_bytes = index.to_be_bytes();

            let addr = match &memory.size {
                NP_Size::U16 => {
                    for x in 0..column_index_bytes.len() {
                        ptr_bytes[4+ x] = column_index_bytes[x];
                    }
            
                    let addr = memory.malloc(ptr_bytes.to_vec())?;
        
                    // set previouse pointer's "next" value to this new pointer
                    let addr_bytes = (addr as u16).to_be_bytes();
                    let memory_write = memory.write_bytes();
                    for x in 0..addr_bytes.len() {
                        memory_write[(curr_addr + 2 + x)] = addr_bytes[x];
                    }
        
                    self.set_tail(addr);
                    addr
                },
                NP_Size::U32 => {
                    for x in 0..column_index_bytes.len() {
                        ptr_bytes[8 + x] = column_index_bytes[x];
                    }
            
                    let addr = memory.malloc(ptr_bytes.to_vec())?;
        
                    // set previouse pointer's "next" value to this new pointer
                    let addr_bytes = addr.to_be_bytes();
                    let memory_write = memory.write_bytes();
                    for x in 0..addr_bytes.len() {
                        memory_write[(curr_addr + 4 + x)] = addr_bytes[x];
                    }
        
                    self.set_tail(addr);
                    addr
                }
            };

            // provide 
            return Ok(NP_Ptr::_new_list_item_ptr(addr as u32, list_of, memory));
        }
    }
/*
    pub fn debug<F>(&self, mut callback: F) -> Result<bool, NP_Error> where F: FnMut(u16, u32, u32) {
        callback(0, self.address, self.head);

        let mut curr_addr = self.head as usize;

        let mut do_continue = true;

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        while do_continue {

            let ptr_index: u16;

            let index_bytes: [u8; 2];

            match memory.get_2_bytes(curr_addr + 8) {
                Some(x) => {
                    index_bytes = *x;
                },
                None => {
                    return Err(NP_Error::new("Out of range request"));
                }
            }

            ptr_index = u16::from_be_bytes(index_bytes);
            

            let next_bytes: [u8; 4] = *memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4]);
            let next_ptr = u32::from_be_bytes(next_bytes) as usize;
            callback(ptr_index, curr_addr as u32, next_ptr as u32);
            if next_ptr == 0 { // out of values to check
                do_continue = false;
            } else {
                // set next pointer for next loop
                curr_addr = next_ptr;
            }
        }

        callback(0, self.address, self.tail);

        Ok(true)
    }
*/

    /// Deletes a value from the list, including it's pointer.
    pub fn delete(&mut self, index: u16) -> Result<bool, NP_Error> {
        if self.head == 0 { // no values in list

            Ok(false)

        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0u32;

            let mut do_continue = true;

            let memory = match &self.memory {
                Some(x) => Rc::clone(x),
                None => unreachable!()
            };

            while do_continue {

                let ptr_index: u16;

                let index_bytes: [u8; 2];

                match memory.get_2_bytes(curr_addr + 8) {
                    Some(x) => {
                        index_bytes = *x;
                    },
                    None => {
                        return Err(NP_Error::new("Out of range request"));
                    }
                }

                ptr_index = u16::from_be_bytes(index_bytes);

                // found our value!
                if ptr_index == index {

                    match memory.size {
                        NP_Size::U16 => {
                            let next_pointer_bytes: [u8; 2];

                            match memory.get_2_bytes(curr_addr + 2) {
                                Some(x) => {
                                    next_pointer_bytes = *x;
                                },
                                None => {
                                    return Err(NP_Error::new("Out of range request"));
                                }
                            }
        
                            if curr_addr == self.head as usize { // item is HEAD
                                self.set_head(u16::from_be_bytes(next_pointer_bytes) as u32);
                            } else { // item is NOT head
                        
                                let memory_bytes = memory.write_bytes();
                        
                                for x in 0..next_pointer_bytes.len() {
                                    memory_bytes[(prev_addr + x as u32 + 2) as usize] = next_pointer_bytes[x as usize];
                                }
                            }
        
                            if curr_addr as u32 == self.tail { // item is tail
                                self.set_tail(prev_addr)
                            }
                        },
                        NP_Size::U32 => {
                            let next_pointer_bytes: [u8; 4];

                            match memory.get_4_bytes(curr_addr + 4) {
                                Some(x) => {
                                    next_pointer_bytes = *x;
                                },
                                None => {
                                    return Err(NP_Error::new("Out of range request"));
                                }
                            }
        
                            if curr_addr == self.head as usize { // item is HEAD
                                self.set_head(u32::from_be_bytes(next_pointer_bytes));
                            } else { // item is NOT head
                        
                                let memory_bytes = memory.write_bytes();
                        
                                for x in 0..next_pointer_bytes.len() {
                                    memory_bytes[(prev_addr + x as u32 + 4) as usize] = next_pointer_bytes[x as usize];
                                }
                            }
        
                            if curr_addr as u32 == self.tail { // item is tail
                                self.set_tail(prev_addr)
                            }
                        }
                    };

                    return Ok(true);
                }

                if ptr_index > index {
                    return Ok(false);
                }

                let next_ptr = match &memory.size {
                    NP_Size::U16 => {
                        u16::from_be_bytes(*memory.get_2_bytes(curr_addr + 2).unwrap_or(&[0; 2])) as usize
                    },
                    NP_Size::U32 => {
                        u32::from_be_bytes(*memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4])) as usize
                    }
                };
                if next_ptr == 0 { // out of values to check
                    do_continue = false;
                } else {
                    // store old value for next loop
                    prev_addr = curr_addr as u32;

                    // set next pointer for next loop
                    curr_addr = next_ptr;
                }
            }

            // ran out of pointers to check, make one!
            Ok(false)
        }
    }

    /// Get the length of the list.  This is NOT the number of items in the list, but the highest index of the last item in the list.
    pub fn len(&self) -> u16 {
        if self.tail == 0 { return 0u16; }

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let tail_index = match &memory.size {
            NP_Size::U16 => *memory.get_2_bytes((self.tail + 4) as usize).unwrap_or(&[0; 2]),
            NP_Size::U32 => *memory.get_2_bytes((self.tail + 8) as usize).unwrap_or(&[0; 2])
        };

        u16::from_be_bytes(tail_index)
    }

    /// Remove the first item from the list and provides it.
    /// 
    /// This returns None once the list is empty.
    /// 
    pub fn shift(&mut self) -> Result<Option<(Option<T>, u16)>, NP_Error> {

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let list_of = match &self.of {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        // no more values in this list
        if self.head == 0 { return Ok(None) }
    
        let index_address_bytes = *memory.get_2_bytes((self.head + 8) as usize).unwrap_or(&[0; 2]);

        let value_address = match &memory.size {
            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(self.head as usize).unwrap_or(&[0; 2])) as u32,
            NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(self.head as usize).unwrap_or(&[0; 4]))
        };

        let next_address = match &memory.size {
            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes((self.head + 2) as usize).unwrap_or(&[0; 2])) as u32,
            NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes((self.head + 4) as usize).unwrap_or(&[0; 4]))
        };

        let index = u16::from_be_bytes(index_address_bytes);

        self.set_head(next_address);

        if self.head == 0 {
            self.set_tail(0);
        }

        // no value for sure
        if value_address == 0 { return Ok(Some((None, index))) }

        let kind = NP_PtrKinds::ListItem { addr: value_address, next: next_address, i: index };

        // try to get the value
        match T::into_value(NP_Lite_Ptr::new_standard(value_address, list_of, memory)) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        Ok(Some((Some(*y), index)))
                    },
                    None => {
                        Ok(Some((None, index)))
                    }
                }
            },
            Err(e) => {
                Err(e)
            }
        }
    }

    /// Push a new value onto the back of the list
    pub fn push(&mut self) -> core::result::Result<(NP_Ptr<T>, u16), NP_Error> {

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let list_of = match &self.of {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        if self.tail == 0 { // no values, create one
       
            let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::ListItem { addr: 0, i: 0, next: 0 }); // List item pointer

            // set index in pointer
            match memory.size {
                NP_Size::U16 => {
                    for x in 0..0u16.to_be_bytes().len() {
                        ptr_bytes[x + 4] = 0;
                    }
                },
                NP_Size::U32 => {
                    for x in 0..0u32.to_be_bytes().len() {
                        ptr_bytes[x + 8] = 0;
                    }
                }
            };


            let addr = memory.malloc(ptr_bytes)?;

            // update head to point to newly created ListItem pointer
            self.set_head(addr);
            self.set_tail(addr);
            
            // provide 
            return Ok((NP_Ptr::_new_list_item_ptr(self.head, list_of, memory), 0));

        } else { 
 
            let tail_addr = self.tail;

            let tail_index_bytes = match &memory.size {
                NP_Size::U32 => *memory.get_2_bytes((tail_addr + 8) as usize).unwrap_or(&[0; 2]),
                NP_Size::U16 => *memory.get_2_bytes((tail_addr + 4) as usize).unwrap_or(&[0; 2]),
            };

            if (u16::from_be_bytes(tail_index_bytes) + 1) as u32 > core::u16::MAX as u32 {
                return Err(NP_Error::new("Error pushing list, out of space!"));
            }

            let new_index = u16::from_be_bytes(tail_index_bytes) + 1;

            let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::ListItem { addr: 0, i: 0, next: 0 }); // List item pointer

            let mult = match &memory.size {
                NP_Size::U32 => 4,
                NP_Size::U16 => 2,
            } as u32;

            // set index in new pointer
            let index_bytes = new_index.to_be_bytes();

            for x in 0..index_bytes.len() {
                ptr_bytes[x + (mult as usize * 2)] = index_bytes[x];
            }

            // set old tail pointer's NEXT to point to new tail pointer
            let addr = memory.malloc(ptr_bytes.to_vec())?;

            let next_addr_bytes = match &memory.size {
                NP_Size::U32 => addr.to_be_bytes().to_vec(),
                NP_Size::U16 => (addr as u16).to_be_bytes().to_vec(),
            };
            
            let memory_write = memory.write_bytes();
            for x in 0..next_addr_bytes.len() {
                memory_write[(tail_addr + mult) as usize + x] = next_addr_bytes[x];
            }

            self.set_tail(addr);

            return Ok((NP_Ptr::_new_list_item_ptr(addr, list_of, memory), new_index));
        }
    }

    /// Check to see if a value exists in the list.
    pub fn has(&self, index: u16) -> Result<bool, NP_Error> {

        if self.head == 0 { // no values in list
            Ok(false)
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;

            let mut do_continue = true;

            let memory = match &self.memory {
                Some(x) => Rc::clone(x),
                None => unreachable!()
            };

            while do_continue {

                let ptr_index: u16;

                let index_bytes: [u8; 2];

                match memory.get_2_bytes(curr_addr + 8) {
                    Some(x) => {
                        index_bytes = *x;
                    },
                    None => {
                        return Err(NP_Error::new("Out of range request"));
                    }
                }

                ptr_index = u16::from_be_bytes(index_bytes);

                // found our value!
                if ptr_index == index {
                    return Ok(true);
                }

                // not found yet, get next address

                let next_ptr = match &memory.size {
                    NP_Size::U16 => {
                        u16::from_be_bytes(*memory.get_2_bytes(curr_addr + 2).unwrap_or(&[0; 2])) as usize
                    },
                    NP_Size::U32 => {
                        u32::from_be_bytes(*memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4])) as usize
                    }
                };
                if next_ptr == 0 { // out of values to check
                    do_continue = false;
                } else {
                    // set next pointer for next loop
                    curr_addr = next_ptr;
                }
            }
            return Ok(false);
        }
    }

    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let addr_bytes = match &memory.size {
            NP_Size::U32 => addr.to_be_bytes().to_vec(),
            NP_Size::U16 => (addr as u16).to_be_bytes().to_vec(),
        };

        let memory_bytes = memory.write_bytes();

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
    }

    fn set_tail(&mut self, addr: u32) {

        self.tail = addr;

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let addr_bytes = match &memory.size {
            NP_Size::U32 => addr.to_be_bytes().to_vec(),
            NP_Size::U16 => (addr as u16).to_be_bytes().to_vec(),
        };


        let memory_bytes = memory.write_bytes();

        let offset = match &memory.size {
            NP_Size::U32 => 4,
            NP_Size::U16 => 2,
        };

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32 + offset) as usize] = addr_bytes[x as usize];
        }
    }
}

impl<T: NP_Value + Default> Default for NP_List<T> {

    fn default() -> Self {
        NP_List { address: 0, head: 0, tail: 0, memory: None, of: None, _value: T::default()}
    }
}

impl<T: NP_Value + Default> NP_Value for NP_List<T> {
    fn is_type( _type_str: &str) -> bool { // not needed for collection types
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (NP_TypeKeys::List as i64, "list".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::List as i64, "list".to_owned()) }
    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        
        match &*ptr.schema.kind {
            NP_SchemaKinds::List { of } => {

                let mut addr = ptr.kind.get_value_addr(); // get pointer of list (head/tail)

                match &ptr.memory.size {
                    NP_Size::U16 => {
                        let mut head: [u8; 2] = [0; 2];
                        let mut tail: [u8; 2] = [0; 2];

        
                        if addr == 0 {
                            // no list here, make one
                            addr = ptr.memory.malloc([0u8; 4].to_vec())?; // stores HEAD & TAIL for list
                            ptr.memory.set_value_address(ptr.location, addr, &ptr.kind);
                        } else {
                            // existing head, read value
                            let a = addr as usize;
                            head = *ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2]);
                            tail = *ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2]);
                        }

        
                        Ok(Some(Box::new(NP_List::<T>::new(addr, u16::from_be_bytes(head) as u32, u16::from_be_bytes(tail) as u32, ptr.memory, Rc::clone(of ))?)))
                    },
                    NP_Size::U32 => {
                        let mut head: [u8; 4] = [0; 4];
                        let mut tail: [u8; 4] = [0; 4];
        
                        if addr == 0 {
                            // no list here, make one
                            addr = ptr.memory.malloc([0u8; 8].to_vec())?; // stores HEAD & TAIL for list
                            ptr.memory.set_value_address(ptr.location, addr, &ptr.kind);
                        } else {
                            // existing head, read value
                            let a = addr as usize;
                            head = *ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]);
                            tail = *ptr.memory.get_4_bytes(a + 4).unwrap_or(&[0; 4]);
                        }
        
                        Ok(Some(Box::new(NP_List::<T>::new(addr, u32::from_be_bytes(head), u32::from_be_bytes(tail), ptr.memory, Rc::clone(of ))?)))
                    }
                }
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        // head + tail;,
        let base_size = match ptr.memory.size {
            NP_Size::U32 => 8u32,
            NP_Size::U16 => 4u32
        };

        match &*ptr.schema.kind {
            NP_SchemaKinds::List { of } => {

                let addr = ptr.kind.get_value_addr();

                if addr == 0 {
                    return Ok(0);
                }

                // existing head, read value
                let a = addr as usize;
                let head = match &ptr.memory.size {
                    NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2])) as u32,
                    NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]))
                };
                let tail = match &ptr.memory.size {
                    NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2])) as u32,
                    NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a + 4).unwrap_or(&[0; 4]))
                };
            
                let list = NP_List::<T>::new(addr, head, tail, ptr.memory, Rc::clone(of) ).unwrap();

                let mut acc_size = 0u32;

                for mut l in list.it().into_iter() {
                    if l.has_value.1 == true {
                        let ptr = l.select()?;
                        acc_size += ptr.calc_size()?;
                    }
                }

                Ok(acc_size + base_size)
            },
            _ => {
                unreachable!();
            }
        }
    }
    
    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {

        match &*ptr.schema.kind {
            NP_SchemaKinds::List { of } => {

                let addr = ptr.kind.get_value_addr();

                if addr == 0 {
                    return NP_JSON::Null;
                }

                let a = addr as usize;
                let head = match &ptr.memory.size {
                    NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2])) as u32,
                    NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]))
                };
                let tail = match &ptr.memory.size {
                    NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2])) as u32,
                    NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a + 4).unwrap_or(&[0; 4]))
                };

                let list = NP_List::<T>::new(addr, head, tail, ptr.memory, Rc::clone(of) ).unwrap_or(NP_List::default());

                let mut json_list = Vec::new();

                for mut l in list.it().into_iter() {
                    if l.has_value.1 == true && l.has_value.0 == true {
                        let ptr = l.select();
                        match ptr {
                            Ok(p) => {
                                json_list.push(p.json_encode());
                            },
                            Err (_e) => {
                                json_list.push(NP_JSON::Null);
                            }
                        }
                    } else {
                        json_list.push(NP_JSON::Null);
                        
                        /*match &schema.default.as_ref().unwrap_or(&NP_JSON::Null) {
                            NP_JSON::True => {
                                match ptr {
                                    Ok(x) => {
                                        json_list.push(x.json_encode());
                                    },
                                    _ => {}
                                };
                            },
                            _ => {
                                json_list.push(NP_JSON::Null);
                            }
                        }*/
                        
                    }
                }

                NP_JSON::Array(json_list)
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn do_compact(from_ptr: NP_Lite_Ptr, to_ptr: NP_Lite_Ptr) -> Result<(), NP_Error> where Self: NP_Value + Default {

        if from_ptr.location == 0 {
            return Ok(());
        }

        let to_ptr_list = to_ptr.into::<NP_List<NP_Any>>();

        let new_address = to_ptr_list.location;

        match Self::into_value(from_ptr)? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(mut new_list) => {

                        for mut item in old_list.it().into_iter() {
                            if item.has_value.0 && item.has_value.1 {

                                let new_ptr = NP_Lite_Ptr::from(new_list.select(item.index)?);
                                let old_ptr = NP_Lite_Ptr::from(item.select()?);
                                old_ptr.compact(new_ptr)?;
                            }
                        }
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok(())
    }
}

/// The iterator type for lists
#[derive(Debug)]
pub struct NP_List_Iterator<T> {
    address: u32, // pointer location
    head: u32,
    tail: u32,
    memory: Rc<NP_Memory>,
    of: Rc<NP_Schema>,
    current_index: u16,
    current_address: u32,
    list: NP_List<T>
}

impl<T: NP_Value + Default + > NP_List_Iterator<T> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, tail: u32, memory: Rc<NP_Memory>, of: Rc<NP_Schema>) -> Self {
        NP_List_Iterator {
            address,
            head,
            tail,
            memory: Rc::clone(&memory),
            of: Rc::clone(&of),
            current_index: 0,
            current_address: head,
            list: NP_List::new(address, head, tail, memory, of).unwrap()
        }
    }
    /// Convert the iterator back into a list
    pub fn into_list(self) -> NP_List<T> {
        self.list
    }
}

impl<T: NP_Value + Default + > Iterator for NP_List_Iterator<T> {
    type Item = NP_List_Item<T>;

    fn next(&mut self) -> Option<Self::Item> {

        if self.current_address == 0 {
            return None;
        }

        let offset = match self.memory.size {
            NP_Size::U16 => 4,
            NP_Size::U32 => 8
        };

        let ptr_index: u16 = u16::from_be_bytes(*self.memory.get_2_bytes((self.current_address + offset) as usize).unwrap_or(&[0; 2]));

        if ptr_index == self.current_index { // pointer matches current index

            let value_address = match &self.memory.size {
                NP_Size::U16 => u16::from_be_bytes(*self.memory.get_2_bytes(self.current_address as usize).unwrap_or(&[0; 2])) as u32,
                NP_Size::U32 => u32::from_be_bytes(*self.memory.get_4_bytes(self.current_address as usize).unwrap_or(&[0; 4]))
            };

            let this_address = self.current_address;
            // point to next value
            self.current_address = match &self.memory.size {
                NP_Size::U16 => u16::from_be_bytes(*self.memory.get_2_bytes((self.current_address + 2) as usize).unwrap_or(&[0; 2])) as u32,
                NP_Size::U32 => u32::from_be_bytes(*self.memory.get_4_bytes((self.current_address + 4) as usize).unwrap_or(&[0; 4]))
            };
            
            self.current_index += 1;
            return Some(NP_List_Item {
                index: self.current_index - 1,
                has_value: (true, value_address != 0),
                of: Rc::clone(&self.of),
                address: this_address,
                list: NP_List::new(self.address, self.head, self.tail, Rc::clone(&self.memory), Rc::clone(&self.of)).unwrap(),
                memory: Rc::clone(&self.memory)
            });

        } else if ptr_index > self.current_index { // pointer is above current index, loop through empty values
            self.current_index += 1;
            return Some(NP_List_Item {
                index: self.current_index - 1,
                has_value: (false, false),
                of: Rc::clone(&self.of),
                address: 0,
                list: NP_List::new(self.address, self.head, self.tail, Rc::clone(&self.memory), Rc::clone(&self.of)).unwrap(),
                memory: Rc::clone(&self.memory)
            });
        }

        None
    }
}

/// A single iterator item
#[derive(Debug)]
pub struct NP_List_Item<T> {
    /// The index of this item in the list
    pub index: u16,
    /// (has pointer at this index, his value at this index)
    pub has_value: (bool, bool),
    of: Rc<NP_Schema>,
    address: u32,
    list: NP_List<T>,
    memory: Rc<NP_Memory>
}

impl<T: NP_Value + Default + > NP_List_Item<T> {
    /// Select the pointer at this item
    pub fn select(&mut self) -> Result<NP_Ptr<T>, NP_Error> {
        self.list.select(self.index)
    }
    /// Delete the pointer and it's value at this item
    pub fn delete(&mut self) -> Result<bool, NP_Error> {
        self.list.delete(self.index)
    }
}
