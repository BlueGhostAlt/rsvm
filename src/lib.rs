#![feature(ptr_internals)]

use std::io::{self, Read, Write};
use std::process::Command;

fn nop(_vm: &mut VM) {}

fn exit(vm: &mut VM) {
    vm.flags.set(Flag::Stop, true);
}

fn syscall(vm: &mut VM) {
    match vm.regs[Register::A as usize] {
        0 => {
            let ptr = vm.regs[1];
            let len = vm.regs[2];

            let mut buf = Vec::with_capacity(len * 4);
            let mut stdout = io::stdout();

            for i in 0..len {
                let value = vm.heap.read(ptr + i).to_be_bytes();
                buf[i * 4] = value[0];
                buf[i * 4 + 1] = value[1];
                buf[i * 4 + 2] = value[2];
                buf[i * 4 + 3] = value[3];
            }

            if stdout.write(&buf).is_err() {
                panic!("Could not proceed with syscall to write to stdout!");
            }
        }
        1 => {
            let ptr = vm.regs[1];
            let len = vm.regs[2];

            let mut buf = Vec::with_capacity(len);
            let mut stdin = io::stdin();

            if stdin.read(&mut buf).is_err() {
                panic!("Could not proceed with syscall to read from stdin!");
            }

            for (i, value) in buf.iter().enumerate() {
                vm.heap.write(ptr + i, *value as u32);
            }
        }
        2 => {
            let command = if cfg!(windows) { "cls" } else { "clear" };

            if Command::new(command).output().is_err() {
                panic!("Could not proceed with syscall to clear screen!");
            }
        }
        3 => println!("{}", vm.regs[Register::B as usize]),
        _ => {}
    }
}

fn push_lit(vm: &mut VM) {
    let lit = vm.fetch_lit();
    vm.stack.push(lit);
}

fn push_reg(vm: &mut VM) {
    let reg = vm.fetch_reg();
    vm.stack.push(vm.regs[reg as usize] as u32);
}

fn pop_reg(vm: &mut VM) {
    let (reg, value) = (vm.fetch_reg(), vm.stack.pop());

    match value {
        Some(value) => vm.regs[reg as usize] = value as usize,
        None => panic!("Could not pop the stack as it's empty!"),
    }
}

fn pop_heap(vm: &mut VM) {
    let (addr, value) = (vm.fetch_lit(), vm.stack.pop());

    match value {
        Some(value) => vm.heap.write(addr as usize, value),
        None => panic!("Could not pop the stack as it's empty!"),
    }
}

fn stack_dupe(vm: &mut VM) {
    vm.stack.push(vm.stack.peek());
}

fn mov_lit_reg(vm: &mut VM) {
    let (reg, value) = (vm.fetch_reg(), vm.fetch_lit());

    vm.regs[reg as usize] = value as usize;
}

fn mov_lit_heap(vm: &mut VM) {
    let addr = vm.fetch_lit();
    let value = vm.fetch_lit();

    vm.heap.write(addr as usize, value);
}

fn mov_heap_reg(vm: &mut VM) {
    let (reg, addr) = (vm.fetch_reg(), vm.fetch_lit());

    vm.regs[reg as usize] = vm.heap.read(addr as usize) as usize;
}

fn mov_reg_heap(vm: &mut VM) {
    let (addr, reg) = (vm.fetch_lit(), vm.fetch_reg());

    vm.heap.write(addr as usize, vm.regs[reg as usize] as u32);
}

fn mov_reg_reg(vm: &mut VM) {
    let (reg_dst, reg_src) = (vm.fetch_reg(), vm.fetch_reg());

    vm.regs[reg_dst as usize] = vm.regs[reg_src as usize];
}

fn mov_heap_heap(vm: &mut VM) {
    let (addr_src, addr_dst) = (vm.fetch_lit(), vm.fetch_lit());

    let value = vm.heap.read(addr_src as usize);
    vm.heap.write(addr_dst as usize, value);
}

fn push_heap(vm: &mut VM) {
    let addr = vm.fetch_lit();
    let value = vm.heap.read(addr as usize);

    vm.stack.push(value);
}

fn math_add_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());

    if vm.regs[a as usize] > usize::MAX - vm.regs[b as usize] {
        vm.flags.set(Flag::Overflow, true);
    }

    vm.regs[a as usize] += vm.regs[b as usize];
}

fn math_add_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => {
            if a > u32::MAX - b {
                vm.flags.set(Flag::Overflow, true);
            }

            vm.stack.push(a + b);
        }
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_sub_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());

    if vm.regs[a as usize] < vm.regs[b as usize] {
        vm.flags.set(Flag::Overflow, true);
    }

    vm.regs[a as usize] -= vm.regs[b as usize];
}

fn math_sub_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => {
            if a < b {
                vm.flags.set(Flag::Overflow, true);
            }

            vm.stack.push(a - b);
        }
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_mul_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());

    if vm.regs[a as usize] > usize::MAX / vm.regs[b as usize] {
        vm.flags.set(Flag::Overflow, true);
    }

    vm.regs[a as usize] *= vm.regs[b as usize];
}

fn math_mul_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => {
            if a > u32::MAX / b {
                vm.flags.set(Flag::Overflow, true);
            }

            vm.stack.push(a * b);
        }
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_div_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());

    vm.regs[a as usize] /= vm.regs[b as usize];
}

fn math_div_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => {
            vm.stack.push(a / b);
        }
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_not_reg(vm: &mut VM) {
    let reg = vm.fetch_reg();
    vm.regs[reg as usize] = !vm.regs[reg as usize];
}

fn math_not_stack(vm: &mut VM) {
    let value = vm.stack.pop();

    match value {
        Some(value) => vm.stack.push(!value),
        None => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_and_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());
    vm.regs[a as usize] &= vm.regs[b as usize];
}

fn math_and_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => vm.stack.push(a & b),
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_or_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());
    vm.regs[a as usize] |= vm.regs[b as usize];
}

fn math_or_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => vm.stack.push(a | b),
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn math_xor_reg(vm: &mut VM) {
    let (a, b) = (vm.fetch_reg(), vm.fetch_reg());
    vm.regs[a as usize] ^= vm.regs[b as usize];
}

fn math_xor_stack(vm: &mut VM) {
    let (a, b) = (vm.stack.pop(), vm.stack.pop());

    match (a, b) {
        (Some(a), Some(b)) => vm.stack.push(a ^ b),
        _ => panic!("Could not pop the stack as it's empty!"),
    }
}

fn compare_reg_reg(vm: &mut VM) {
    let (a, b) = (
        vm.regs[vm.fetch_reg() as usize],
        vm.regs[vm.fetch_reg() as usize],
    );

    vm.compare_numbers(a as u32, b as u32);
}

fn compare_reg_lit(vm: &mut VM) {
    let (a, b) = (
        vm.regs[vm.fetch_reg() as usize],
        vm.fetch_lit(),
    );

    vm.compare_numbers(a as u32, b);
}

fn compare_stack_lit(vm: &mut VM) {
    let (a, b) = (
        vm.stack.peek(),
        vm.fetch_lit(),
    );

    vm.compare_numbers(a, b);
}

fn jump_absolute(vm: &mut VM) {
    let addr = vm.fetch_lit();
    vm.prgrm_cntr = (addr as usize) - 1;
}

fn jump_equal(vm: &mut VM) {
    let addr = vm.fetch_lit();

    if vm.flags.get(Flag::Equal) {
        vm.prgrm_cntr = (addr as usize) - 1;
    }
}

fn jump_not_equal(vm: &mut VM) {
    let addr = vm.fetch_lit();

    if vm.flags.get(Flag::NotEqual) {
        vm.prgrm_cntr = (addr as usize) - 1;
    }
}

fn jump_greater(vm: &mut VM) {
    let addr = vm.fetch_lit();

    if vm.flags.get(Flag::Greater) {
        vm.prgrm_cntr = (addr as usize) - 1;
    }
}

fn jump_smaller(vm: &mut VM) {
    let addr = vm.fetch_lit();

    if vm.flags.get(Flag::Smaller) {
        vm.prgrm_cntr = (addr as usize) - 1;
    }
}

fn jump_overflow(vm: &mut VM) {
    let addr = vm.fetch_lit();

    if vm.flags.get(Flag::Overflow) {
        vm.prgrm_cntr = (addr as usize) - 1;
    }
}

const OP_CODES: [fn(&mut VM); 256] = [
    exit,              // 0x00
    push_lit,          // 0x01
    push_reg,          // 0x02
    pop_reg,           // 0x03
    pop_heap,          // 0x04
    stack_dupe,        // 0x05
    mov_lit_reg,       // 0x06
    mov_lit_heap,      // 0x07
    mov_heap_reg,      // 0x08
    mov_reg_heap,      // 0x09
    mov_reg_reg,       // 0x0A
    mov_heap_heap,     // 0x0B
    push_heap,         // 0x0C
    nop,               // 0x0D
    nop,               // 0x0E
    nop,               // 0x0F
    math_add_reg,      // 0x10
    math_add_stack,    // 0x11
    math_sub_reg,      // 0x12
    math_sub_stack,    // 0x13
    math_mul_reg,      // 0x14
    math_mul_stack,    // 0x15
    math_div_reg,      // 0x16
    math_div_stack,    // 0x17
    math_not_reg,      // 0x18
    math_not_stack,    // 0x19
    math_and_reg,      // 0x1A
    math_and_stack,    // 0x1B
    math_or_reg,       // 0x1C
    math_or_stack,     // 0x1D
    math_xor_reg,      // 0x1E
    math_xor_stack,    // 0x1F
    jump_absolute,     // 0x20
    nop,               // 0x21
    nop,               // 0x22
    nop,               // 0x23
    nop,               // 0x24
    nop,               // 0x25
    nop,               // 0x26
    nop,               // 0x27
    nop,               // 0x28
    nop,               // 0x29
    nop,               // 0x2A
    nop,               // 0x2B
    nop,               // 0x2C
    nop,               // 0x2D
    nop,               // 0x2E
    nop,               // 0x2F
    compare_reg_reg,   // 0x30
    compare_reg_lit,   // 0x31
    compare_stack_lit, // 0x32
    jump_equal,        // 0x33
    jump_not_equal,    // 0x34
    jump_greater,      // 0x35
    jump_smaller,      // 0x36
    jump_overflow,     // 0x37
    nop,               // 0x38
    nop,               // 0x39
    nop,               // 0x3A
    nop,               // 0x3B
    nop,               // 0x3C
    nop,               // 0x3D
    nop,               // 0x3E
    nop,               // 0x3F
    nop,               // 0x40
    nop,               // 0x41
    nop,               // 0x42
    nop,               // 0x43
    nop,               // 0x44
    nop,               // 0x45
    nop,               // 0x46
    nop,               // 0x47
    nop,               // 0x48
    nop,               // 0x49
    nop,               // 0x4A
    nop,               // 0x4B
    nop,               // 0x4C
    nop,               // 0x4D
    nop,               // 0x4E
    nop,               // 0x4F
    nop,               // 0x50
    nop,               // 0x51
    nop,               // 0x52
    nop,               // 0x53
    nop,               // 0x54
    nop,               // 0x55
    nop,               // 0x56
    nop,               // 0x57
    nop,               // 0x58
    nop,               // 0x59
    nop,               // 0x5A
    nop,               // 0x5B
    nop,               // 0x5C
    nop,               // 0x5D
    nop,               // 0x5E
    nop,               // 0x5F
    nop,               // 0x60
    nop,               // 0x61
    nop,               // 0x62
    nop,               // 0x63
    nop,               // 0x64
    nop,               // 0x65
    nop,               // 0x66
    nop,               // 0x67
    nop,               // 0x68
    nop,               // 0x69
    nop,               // 0x6A
    nop,               // 0x6B
    nop,               // 0x6C
    nop,               // 0x6D
    nop,               // 0x6E
    nop,               // 0x6F
    nop,               // 0x70
    nop,               // 0x71
    nop,               // 0x72
    nop,               // 0x73
    nop,               // 0x74
    nop,               // 0x75
    nop,               // 0x76
    nop,               // 0x77
    nop,               // 0x78
    nop,               // 0x79
    nop,               // 0x7A
    nop,               // 0x7B
    nop,               // 0x7C
    nop,               // 0x7D
    nop,               // 0x7E
    nop,               // 0x7F
    nop,               // 0x80
    nop,               // 0x81
    nop,               // 0x82
    nop,               // 0x83
    nop,               // 0x84
    nop,               // 0x85
    nop,               // 0x86
    nop,               // 0x87
    nop,               // 0x88
    nop,               // 0x89
    nop,               // 0x8A
    nop,               // 0x8B
    nop,               // 0x8C
    nop,               // 0x8D
    nop,               // 0x8E
    nop,               // 0x8F
    nop,               // 0x90
    nop,               // 0x91
    nop,               // 0x92
    nop,               // 0x93
    nop,               // 0x94
    nop,               // 0x95
    nop,               // 0x96
    nop,               // 0x97
    nop,               // 0x98
    nop,               // 0x99
    nop,               // 0x9A
    nop,               // 0x9B
    nop,               // 0x9C
    nop,               // 0x9D
    nop,               // 0x9E
    nop,               // 0x9F
    nop,               // 0xA0
    nop,               // 0xA1
    nop,               // 0xA2
    nop,               // 0xA3
    nop,               // 0xA4
    nop,               // 0xA5
    nop,               // 0xA6
    nop,               // 0xA7
    nop,               // 0xA8
    nop,               // 0xA9
    nop,               // 0xAA
    nop,               // 0xAB
    nop,               // 0xAC
    nop,               // 0xAD
    nop,               // 0xAE
    nop,               // 0xAF
    nop,               // 0xB0
    nop,               // 0xB1
    nop,               // 0xB2
    nop,               // 0xB3
    nop,               // 0xB4
    nop,               // 0xB5
    nop,               // 0xB6
    nop,               // 0xB7
    nop,               // 0xB8
    nop,               // 0xB9
    nop,               // 0xBA
    nop,               // 0xBB
    nop,               // 0xBC
    nop,               // 0xBD
    nop,               // 0xBE
    nop,               // 0xBF
    nop,               // 0xC0
    nop,               // 0xC1
    nop,               // 0xC2
    nop,               // 0xC3
    nop,               // 0xC4
    nop,               // 0xC5
    nop,               // 0xC6
    nop,               // 0xC7
    nop,               // 0xC8
    nop,               // 0xC9
    nop,               // 0xCA
    nop,               // 0xCB
    nop,               // 0xCC
    nop,               // 0xCD
    nop,               // 0xCE
    nop,               // 0xCF
    nop,               // 0xD0
    nop,               // 0xD1
    nop,               // 0xD2
    nop,               // 0xD3
    nop,               // 0xD4
    nop,               // 0xD5
    nop,               // 0xD6
    nop,               // 0xD7
    nop,               // 0xD8
    nop,               // 0xD9
    nop,               // 0xDA
    nop,               // 0xDB
    nop,               // 0xDC
    nop,               // 0xDD
    nop,               // 0xDE
    nop,               // 0xDF
    nop,               // 0xE0
    nop,               // 0xE1
    nop,               // 0xE2
    nop,               // 0xE3
    nop,               // 0xE4
    nop,               // 0xE5
    nop,               // 0xE6
    nop,               // 0xE7
    nop,               // 0xE8
    nop,               // 0xE9
    nop,               // 0xEA
    nop,               // 0xEB
    nop,               // 0xEC
    nop,               // 0xED
    nop,               // 0xEE
    nop,               // 0xEF
    nop,               // 0xF0
    nop,               // 0xF1
    nop,               // 0xF2
    nop,               // 0xF3
    nop,               // 0xF4
    nop,               // 0xF5
    nop,               // 0xF6
    nop,               // 0xF7
    nop,               // 0xF8
    nop,               // 0xF9
    nop,               // 0xFA
    nop,               // 0xFB
    nop,               // 0xFC
    nop,               // 0xFD
    nop,               // 0xFE
    syscall,           // 0xFF
];

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
        if value == 0 {
            return;
        }
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

pub enum Register {
    A,
    B,
    C,
    D,
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
    pub regs: [usize; 4],
    pub flags: FlagSet,
    stack: Stack,
    pub heap: Heap,
    bytecode: Vec<u8>,
    pub prgrm_cntr: usize,
    base_ptr: u32,
    hdr_size: usize,
}

impl VM {
    fn is_at_end_header(&self, i: usize) -> bool {
        self.bytecode[i..i + 4].iter().all(|nibble| nibble == &0x1d)
    }

    fn parse_header(&mut self) {
        self.hdr_size = 4;

        for i in 0..=self.bytecode.len() - 4 {
            if self.is_at_end_header(i) {
                break;
            }

            self.heap.write(i, self.bytecode[i] as u32);
            self.hdr_size += 1;
        }
    }

    fn step_program(&mut self) {
        let instruction = self.bytecode[self.prgrm_cntr + self.hdr_size];

        OP_CODES[instruction as usize](self);
    }

    pub fn new() -> VM {
        Default::default()
    }

    pub fn load_program(&mut self, bytecode: Vec<u8>) {
        self.bytecode = bytecode;
    }

    pub fn run_program(&mut self) {
        self.parse_header();

        while !self.flags.get(Flag::Stop) {
            self.step_program();
            self.prgrm_cntr += 1;
        }
    }

    pub fn fetch_reg(&mut self) -> u8 {
        self.prgrm_cntr += 1;

        self.bytecode[self.prgrm_cntr + self.hdr_size]
    }

    pub fn fetch_lit(&mut self) -> u32 {
        let regs = (
            self.fetch_reg() as u32,
            self.fetch_reg() as u32,
            self.fetch_reg() as u32,
            self.fetch_reg() as u32,
        );

        (regs.0 << 24) + (regs.1 << 16) + (regs.2 << 8) + regs.3
    }

    pub fn compare_numbers(&mut self, a: u32, b: u32) {
        self.flags.set(Flag::Equal, a == b);
        self.flags.set(Flag::NotEqual, a != b);
        self.flags.set(Flag::Greater, a > b);
        self.flags.set(Flag::Smaller, a < b);
    }
}
