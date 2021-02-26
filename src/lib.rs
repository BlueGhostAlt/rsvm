#![feature(ptr_internals)]

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::mem;
use std::process;
use std::ptr::{self, Unique};

const HEAP_INITIAL_CAPACITY: usize = 256; // 1KB
const STACK_INITIAL_CAPACITY: usize = 128; // 512B

const NO_OF_FLAGS: usize = 6;

#[derive(Debug)]
pub enum Flag {
    Equal,
    NotEqual,
    Greater,
    Smaller,
    Overflow,
    Stop,
}

#[derive(Debug)]
pub struct FlagSet([bool; NO_OF_FLAGS]);

impl FlagSet {
    pub fn new() -> FlagSet {
        FlagSet([false; NO_OF_FLAGS])
    }

    pub fn get(&self, flag: Flag) -> bool {
        self.0[flag as usize]
    }

    pub fn set(&mut self, flag: Flag, value: bool) {
        self.0[flag as usize] = value;
    }
}

impl Default for FlagSet {
    fn default() -> FlagSet {
        FlagSet::new()
    }
}

#[derive(Debug)]
pub struct Heap {
    ptr: Unique<u32>,
    cap: usize,
}

impl Heap {
    fn ptr(&self) -> *mut u32 {
        self.ptr.as_ptr()
    }

    fn grow(&mut self, new_cap: usize) {
        unsafe {
            let elem_size = mem::size_of::<u32>();
            let align = mem::align_of::<u32>();

            let ptr = {
                let layout = Layout::from_size_align_unchecked(self.cap * elem_size, align);
                realloc(self.ptr.as_ptr() as *mut _, layout, new_cap * elem_size)
            };

            if ptr.is_null() {
                eprintln!("Failed to reallocate(grow) VM heap! Aborting!");

                process::abort();
            }

            self.ptr = Unique::new_unchecked(ptr as *mut _);
            self.cap = new_cap;
        }
    }

    pub fn new() -> Heap {
        let cap = HEAP_INITIAL_CAPACITY;

        let elem_size = mem::size_of::<u32>();
        let align = mem::align_of::<u32>();
        let ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(cap * elem_size, align);
            alloc(layout)
        };

        if ptr.is_null() {
            eprintln!("Failed to allocate VM heap! Aborting!");

            process::abort();
        }

        let ptr = unsafe { Unique::new_unchecked(ptr as *mut _) };

        Heap { ptr, cap }
    }

    pub fn read(&self, addr: usize) -> u32 {
        if addr > self.cap {
            return 0;
        }

        unsafe { ptr::read(self.ptr().add(addr)) }
    }

    pub fn write(&mut self, addr: usize, value: u32) {
        if addr >= self.cap {
            self.grow(addr + 1);
        }

        unsafe {
            ptr::write(self.ptr().add(addr), value);
        }
    }
}

impl Default for Heap {
    fn default() -> Heap {
        Heap::new()
    }
}

#[derive(Debug)]
struct RawStack {
    ptr: Unique<u32>,
    cap: usize,
}

impl RawStack {
    fn new() -> RawStack {
        let cap = STACK_INITIAL_CAPACITY;

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
                eprintln!("Failed to reallocate(grow) VM stack! Aborting!");

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
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        let elem = unsafe { ptr::read(self.ptr().add(self.len)) };

        Some(elem)
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
    flags: FlagSet,
    stack: Stack,
    heap: Heap,
    bytecode: Vec<u8>,
    prgrm_cntr: i32,
    base_ptr: i32,
    hdr_size: usize,
}

impl VM {
    fn is_at_end_header(&self, i: usize) -> bool {
        self.bytecode[i..i + 4].iter().all(|nibble| nibble == &0x1d)
    }

    pub fn new() -> VM {
        Default::default()
    }

    pub fn load_program(&mut self, bytecode: Vec<u8>) {
        self.bytecode = bytecode;
    }

    pub fn parse_header(&mut self) {
        self.hdr_size = 4;

        for i in 0..=self.bytecode.len() - 4 {
            if self.is_at_end_header(i) {
                break;
            }

            self.heap.write(i, self.bytecode[i] as u32);
            self.hdr_size += 1;
        }
    }
}
