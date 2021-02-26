#![feature(ptr_internals)]

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::mem;
use std::process;
use std::ptr::{self, Unique};

#[derive(Debug)]
struct RawStack {
    ptr: Unique<u32>,
    cap: usize,
}

impl RawStack {
    fn new() -> RawStack {
        let cap = 128;

        let elem_size = mem::size_of::<u32>();
        let align = mem::align_of::<u32>();
        let ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(cap * elem_size, align);
            alloc(layout)
        };

        if ptr.is_null() {
            eprintln!("Failed to allocate VM stack! Aborting!");

            process::abort();
        }

        let ptr = unsafe { Unique::new_unchecked(ptr as *mut _) };

        RawStack { ptr, cap }
    }

    fn grow(&mut self) {
        unsafe {
            let elem_size = mem::size_of::<u32>();
            let align = mem::align_of::<u32>();

            let (new_cap, ptr) = {
                let new_cap = self.cap * 2;

                let layout = Layout::from_size_align_unchecked(self.cap * elem_size, align);
                let ptr = realloc(self.ptr.as_ptr() as *mut _, layout, new_cap * elem_size);

                (new_cap, ptr)
            };

            if ptr.is_null() {
                eprintln!("Failed to grow(reallocate) VM stack! Aborting!");

                process::abort();
            }

            self.ptr = Unique::new_unchecked(ptr as *mut _);
            self.cap = new_cap;
        }
    }
}

impl Drop for RawStack {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<u32>();

        if self.cap != 0 {
            let align = mem::align_of::<u32>();

            unsafe {
                let layout = Layout::from_size_align_unchecked(self.cap * elem_size, align);
                dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    buf: RawStack,
    len: usize,
}

impl Stack {
    fn ptr(&self) -> *mut u32 {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Stack {
        Stack {
            buf: RawStack::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, elem: u32) {
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<u32> {
        match self.len {
            0 => None,
            _ => {
                self.len -= 1;

                unsafe {
                    let elem = ptr::read(self.ptr().add(self.len));

                    Some(elem)
                }
            }
        }
    }

    pub fn peek(&self) -> u32 {
        unsafe { ptr::read(self.ptr().add(self.len)) }
    }

    pub fn clear(&mut self) {
        unsafe {
            let slice = ptr::slice_from_raw_parts_mut(self.ptr(), self.len);
            ptr::drop_in_place(slice);

            self.len = 0;
        }
    }
}

impl Default for Stack {
    fn default() -> Stack {
        Stack::new()
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

#[derive(Debug, Default)]
pub struct VM {
    regs: [i32; 4],
    stack: Stack,
    bytecode: Vec<u8>,
    prgrm_cntr: i32,
    base_ptr: i32,
    hdr_size: usize,
}

impl VM {
    pub fn new() -> VM {
        Default::default()
    }

    pub fn load_program(&mut self, bytecode: Vec<u8>) {
        self.bytecode = bytecode;
    }
}
