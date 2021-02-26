enum Register {
    A,
    B,
    C,
    D,
}

#[derive(Default)]
pub struct VM {
    registers: [i32; 4],
    bytecode: Vec<u8>,
    program_counter: i32,
    base_pointer: i32,
    header_size: usize,
    program_length: usize,
}

impl VM {
    pub fn new() -> VM {
        Default::default()
    }
}
