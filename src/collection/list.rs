use crate::pointer::NP_ValueInto;
use crate::{memory::NP_Memory, pointer::{NP_Value, NP_Ptr, NP_PtrKinds}, error::NP_Error, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
pub struct NP_List<'a, T> {
    address: u32, // pointer location
    head: u32,
    tail: u32,
    memory: Option<&'a NP_Memory>,
    of: Option<&'a NP_Schema>,
    _value: T
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_List<'a, T> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, tail:u32,  memory: &'a NP_Memory, of: &'a NP_Schema) -> Result<Self, NP_Error> {

        // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
        if T::type_idx().0 != NP_TypeKeys::Any as i64 && of.type_data.0 != NP_TypeKeys::Any as i64  {

            // not using any casting, check type
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

    pub fn select(&mut self, index: u16) -> core::result::Result<NP_Ptr<'a, T>, NP_Error> {

        if self.head == 0 { // no values, create one


            let memory = self.memory.unwrap();

            let mut ptr_bytes: [u8; 10] = [0; 10]; // List item pointer

            // set index in pointer
            let index_bytes = index.to_le_bytes();

            for x in 0..index_bytes.len() {
                ptr_bytes[x + 8] = index_bytes[x];
            }

            let addr = memory.malloc(ptr_bytes.to_vec())?;

            // update head to point to newly created ListItem pointer
            self.set_head(addr);
            self.set_tail(addr);
            
            // provide 
            return Ok(NP_Ptr::new_list_item_ptr(self.head, self.of.unwrap(), &memory));

        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0;

            let mut do_continue = true;

            let memory = self.memory.unwrap();

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

                ptr_index = u16::from_le_bytes(index_bytes);

                // found our value!
                if ptr_index == index {
                    return Ok(NP_Ptr::new_list_item_ptr(curr_addr as u32, self.of.unwrap(), &memory));
                }

                // we've found an existing value above the requested index
                // insert a new pointer in before the current one in the loop
                if ptr_index > index {

                    let new_addr = {
            
                        let mut ptr_bytes: [u8; 10] = [0; 10]; // list item pointer

                        // set "next" value of this new pointer to current pointer in the loop
                        let curr_addr_bytes = (curr_addr as u32).to_le_bytes();
                        for x in 0..curr_addr_bytes.len() {
                            ptr_bytes[4 + x] = curr_addr_bytes[x]; 
                        }

                        // set index of the new pointer
                        let index_bytes = index.to_le_bytes();
                        for x in 0..index_bytes.len() {
                            ptr_bytes[8 + x] = index_bytes[x]; 
                        }
    
                        memory.malloc(ptr_bytes.to_vec())?
                    };

                    if curr_addr == self.head as usize {
                        // update head to new pointer
                        self.set_head(new_addr);
                    } else {
                        // update "next" value of previous pointer to the one we just made
                        let new_addr_bytes = new_addr.to_le_bytes();

                        let memory_write = memory.write_bytes();

                        for x in 0..new_addr_bytes.len() {
                            memory_write[prev_addr + 4 + x] = new_addr_bytes[x];
                        }
                    }

                    return Ok(NP_Ptr::new_list_item_ptr(new_addr as u32, self.of.unwrap(), &memory));
                } else {
                    // not found yet, get next address

                    let next_bytes: [u8; 4] = *memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4]);
                    let next_ptr = u32::from_le_bytes(next_bytes) as usize;
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
            let mut ptr_bytes: [u8; 10] = [0; 10]; // list item pointer

            // get index bytes
            let column_index_bytes = index.to_le_bytes();

            for x in 0..column_index_bytes.len() {
                ptr_bytes[8 + x] = column_index_bytes[x];
            }
    
            let addr = memory.malloc(ptr_bytes.to_vec())?;

            // set previouse pointer's "next" value to this new pointer
            let addr_bytes = addr.to_le_bytes();
            let memory_write = memory.write_bytes();
            for x in 0..addr_bytes.len() {
                memory_write[(curr_addr + 4 + x)] = addr_bytes[x];
            }

            self.set_tail(addr);

            // provide 
            return Ok(NP_Ptr::new_list_item_ptr(addr as u32, self.of.unwrap(), &memory));
        }
    }

    pub fn delete(&mut self, index: u16) -> Result<bool, NP_Error> {
        if self.head == 0 { // no values in list

            Ok(false)

        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0u32;

            let mut do_continue = true;

            let memory = self.memory.unwrap();

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

                ptr_index = u16::from_le_bytes(index_bytes);

                // found our value!
                if ptr_index == index {

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
                        self.set_head(u32::from_le_bytes(next_pointer_bytes));
                    } else { // item is NOT head
                
                        let memory_bytes = memory.write_bytes();
                
                        for x in 0..next_pointer_bytes.len() {
                            memory_bytes[(prev_addr + x as u32 + 4) as usize] = next_pointer_bytes[x as usize];
                        }
                    }

                    if curr_addr as u32 == self.tail { // item is tail
                        self.set_tail(prev_addr)
                    }
                    
                    return Ok(true);
                }

                if ptr_index > index {
                    return Ok(false);
                }

                let next_bytes: [u8; 4] = *memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4]);
                let next_ptr = u32::from_le_bytes(next_bytes) as usize;
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

    pub fn push(&mut self) -> core::result::Result<(NP_Ptr<'a, T>, u16), NP_Error> {

        let memory = self.memory.unwrap();

        if self.tail == 0 { // no values, create one
       
            let mut ptr_bytes: [u8; 10] = [0; 10]; // List item pointer

            // set index in pointer
            let index_bytes = 0u32.to_le_bytes();

            for x in 0..index_bytes.len() {
                ptr_bytes[x + 8] = index_bytes[x];
            }

            let addr = memory.malloc(ptr_bytes.to_vec())?;

            // update head to point to newly created ListItem pointer
            self.set_head(addr);
            self.set_tail(addr);
            
            // provide 
            return Ok((NP_Ptr::new_list_item_ptr(self.head, self.of.unwrap(), &memory), 0));

        } else { 
 
            let tail_addr = self.tail;

            let tail_index_bytes = *memory.get_2_bytes((tail_addr + 8) as usize).unwrap_or(&[0; 2]);

            if (u16::from_le_bytes(tail_index_bytes) + 1) as u32 > core::u16::MAX as u32 {
                return Err(NP_Error::new("Error pushing list, out of space!"));
            }

            let new_index = u16::from_le_bytes(tail_index_bytes) + 1;

            let mut ptr_bytes: [u8; 10] = [0; 10]; // List item pointer

            // set index in new pointer
            let index_bytes = new_index.to_le_bytes();

            for x in 0..index_bytes.len() {
                ptr_bytes[x + 8] = index_bytes[x];
            }

            // set old tail pointer's NEXT to point to new tail pointer
            let addr = memory.malloc(ptr_bytes.to_vec())?;

            let next_addr_bytes = addr.to_le_bytes();
            
            let memory_write = memory.write_bytes();
            for x in 0..next_addr_bytes.len() {
                memory_write[(tail_addr + 4) as usize] = next_addr_bytes[x];
            }

            self.set_tail(addr);

            return Ok((NP_Ptr::new_list_item_ptr(addr, self.of.unwrap(), &memory), new_index));
        }
    }

    pub fn has(&self, index: u16) -> Result<bool, NP_Error> {

        if self.head == 0 { // no values in list
            Ok(false)
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;

            let mut do_continue = true;

            let memory = self.memory.unwrap();

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

                ptr_index = u16::from_le_bytes(index_bytes);

                // found our value!
                if ptr_index == index {
                    return Ok(true);
                }

                // not found yet, get next address

                let next_bytes: [u8; 4] = *memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4]);
                let next_ptr = u32::from_le_bytes(next_bytes) as usize;
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

        let memory = self.memory.unwrap();

        let addr_bytes = addr.to_le_bytes();

        let memory_bytes = memory.write_bytes();

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
    }

    fn set_tail(&mut self, addr: u32) {

        self.tail = addr;

        let memory = self.memory.unwrap();

        let addr_bytes = addr.to_le_bytes();

        let memory_bytes = memory.write_bytes();

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32 + 4) as usize] = addr_bytes[x as usize];
        }
    }
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_Value for NP_List<'a, T> {
    fn new<X>() -> Self {
        unreachable!()
    }
    fn is_type( _type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "map".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "map".to_owned()) }
    fn buffer_get(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .get()! Use .into() instead."))
    }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .set()! Use .into() instead."))
    }
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> Default for NP_List<'a, T> {

    fn default() -> Self {
        NP_List { address: 0, head: 0, tail: 0, memory: None, of: None, _value: T::default()}
    }
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_ValueInto<'a> for NP_List<'a, T> {
    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<Option<Box<NP_List<'a, T>>>, NP_Error> {
        
        match &*schema.kind {
            NP_SchemaKinds::List { of } => {

                let mut addr = kind.get_value(); // get pointer of list (head/tail)

                let mut head: [u8; 4] = [0; 4];
                let mut tail: [u8; 4] = [0; 4];

                if addr == 0 {
                    // no list here, make one
                    addr = buffer.malloc([0u8; 8].to_vec())?; // stores HEAD & TAIL for list
                    buffer.set_value_address(address, addr, &kind);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *buffer.get_4_bytes(a).unwrap_or(&[0; 4]);
                    tail = *buffer.get_4_bytes(a + 4).unwrap_or(&[0; 4]);
                }

                Ok(Some(Box::new(NP_List::<T>::new(addr, u32::from_le_bytes(head), u32::from_le_bytes(tail), buffer, of )?)))
            },
            _ => {
                Err(NP_Error::new(""))
            }
        }
    }
}