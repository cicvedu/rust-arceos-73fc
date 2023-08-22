//! Simple memory allocation.
//!
//! TODO: more efficient
use super::{AllocError};
use core::alloc::Layout;
use core::num::NonZeroUsize;
//use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use crate::{AllocResult, BaseAllocator, ByteAllocator};

 const HEAP_SPACE_SIZE:usize =  1024 * 1024 ;



pub struct Node {
    next: usize,
    prev: usize,
    size: usize ,
    start:usize ,
    is_use:bool
}


pub struct SimpleByteAllocator {
    book_array:[Node; HEAP_SPACE_SIZE ],
    total_size:usize ,
    use_size:usize ,
    len: i32
}

impl Copy for Node {}

impl Clone for Node{
    fn clone(&self) -> Node {
        Node { next: self.next, prev: self.prev, size: self.size, start: self.start, is_use : self.is_use}
    }
}

impl Node {
    pub const fn new() -> Self {
        Node { next: HEAP_SPACE_SIZE + 1, prev: 0, size: 0, start: 0, is_use : false }
    }
}


impl SimpleByteAllocator {
    pub const fn new() -> Self {
        Self {book_array:[Node::new(); HEAP_SPACE_SIZE ] , total_size: 0, use_size: 0, len:0}
      // Self { total_size: 0, use_size: 0, len:0}
    }

    fn get_next_node(&self) -> usize {
        for i in (1..HEAP_SPACE_SIZE).rev() {
            
            if HEAP_SPACE_SIZE < self.book_array[i].next {
                return i;
            
            }
        }
        0
    }
}

impl BaseAllocator for SimpleByteAllocator {
    fn init(&mut self, _start: usize, _size: usize) {
        self.total_size = _size;
        self.book_array[0].next = 1;
        self.book_array[0].prev = 1;
        self.book_array[1].next = 0;
        self.book_array[1].prev = 0;
        self.book_array[1].start =  _start;
        self.book_array[1].size = _size;
        self.book_array[1].is_use = false;

  
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {

        let idx = self.book_array[0 as usize].prev;
        if self.book_array[idx].start + self.book_array[idx].size  ==  _start && self.book_array[idx].is_use == false {
            self.book_array[idx].size += self.use_size;
            return AllocResult::Ok(());
        }
        let next_idx = self.get_next_node();
        if next_idx == 0 {
            return AllocResult::Err(AllocError::NoMemory);
        }
        self.book_array[next_idx].prev = idx;
        self.book_array[next_idx].next = self.book_array[idx].next;
        self.book_array[next_idx].size = _size;
        self.book_array[next_idx].start = _start;
        self.total_size  += _size;
        AllocResult::Ok(()) 
    }
}

impl ByteAllocator for SimpleByteAllocator {
    fn alloc(&mut self, _layout: Layout) -> AllocResult<NonZeroUsize> {
        let mut idx:usize = 1;
            loop {
            if  self.book_array[idx].is_use == false && _layout.size() <= self.book_array[idx].size  {
                let rem = (self.book_array[idx].start ) % (_layout.align()  );
                if rem == 0 {
                    if _layout.size() < self.book_array[idx].size {
                        let next_idx = self.get_next_node();
                        self.book_array[next_idx].prev = idx;
                        self.book_array[next_idx].next = self.book_array[idx].next;
                        self.book_array[next_idx].size = self.book_array[idx].size - _layout.size();
                        self.book_array[next_idx].start = self.book_array[idx].start + _layout.size();
                        self.book_array[next_idx].is_use = false; 
                        self.book_array[idx].next = next_idx;
                    }
                    self.book_array[idx].size = _layout.size();
                    self.book_array[idx].is_use = true;
                    self.use_size += _layout.size();
                    return AllocResult::Ok(NonZeroUsize::new(self.book_array[idx].start).unwrap());                    
                } else if rem + _layout.size() <= self.book_array[idx].size {
                    let next_idx = self.get_next_node();
                    self.book_array[next_idx].prev = idx;
                    self.book_array[next_idx].next = self.book_array[idx].next;
                    self.book_array[next_idx].size = _layout.size();
                    self.book_array[next_idx].start = self.book_array[idx].start + rem;
                    self.book_array[next_idx].is_use = true; 
                    if rem + _layout.size() < self.book_array[idx].size {
                        let next_next_idx = self.get_next_node();
                        self.book_array[next_next_idx].prev = next_idx;
                        self.book_array[next_next_idx].next = self.book_array[next_idx].next;
                        self.book_array[next_next_idx].size = self.book_array[idx].size - _layout.size() - rem;
                        self.book_array[next_next_idx].start = self.book_array[next_idx].start + self.book_array[next_idx].size;
                        self.book_array[next_next_idx].is_use = false; 
                        self.book_array[next_idx].next = next_next_idx;
                    }
                    self.book_array[idx].next = next_idx;
                    self.book_array[idx].size = rem;
                    self.use_size += _layout.size();
                    return AllocResult::Ok(NonZeroUsize::new(self.book_array[next_idx].start).unwrap());    
                }
            }

            idx = self.book_array[idx].next;
            if idx == 0 { break; }
        }
        AllocResult::Err(AllocError::NoMemory)
    }

    fn dealloc(&mut self, _pos: NonZeroUsize, _layout: Layout) {
        let mut idx:usize = 1;
        let pos = _pos.get();
        // loop {
        //     if self.book_array[idx].start <= pos && pos <  self.book_array[idx].start  + self.book_array[idx].size  && self.book_array[idx].is_use {
        //         ();
        //     }

        //     idx = self.book_array[idx].next;
        //     if idx == 0 { break; }
        // }
        self.use_size -= _layout.size();
    }

    fn total_bytes(&self) -> usize {
        self.total_size
    }

    fn used_bytes(&self) -> usize {
        self.use_size
    }

    fn available_bytes(&self) -> usize {
        self.total_size - self.use_size
    }
}
