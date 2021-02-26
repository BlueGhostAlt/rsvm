enum Register {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Default)]
pub struct VM {
    registers: [i32; 4],
    bytecode: Vec<u8>,
    program_counter: i32,
    base_pointer: i32,
    header_size: usize,
}

impl VM {
    pub fn new() -> VM {
        Default::default()
    }

    pub fn load_program(&mut self, bytecode: Vec<u8>) {
        self.bytecode = bytecode;
    }
}
