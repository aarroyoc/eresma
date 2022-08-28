#[derive(Debug, Clone)]
pub struct Stack {
    pub st: Vec<u8>, // Stack
    pub p: usize,    // Pointer of the stack
    k: usize,        // Keep Mode relative pointer
    keep_mode: bool,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            st: vec![0; 256],
            p: 0x00,
            k: 1,
            keep_mode: false,
        }
    }

    pub fn set_current_opcode(&mut self, opcode: u8) {
        self.k = 1; // reset keep mode relative pointer
        if opcode < 0x80 {
            self.keep_mode = false;
        } else {
            self.keep_mode = true;
        }
    }

    pub fn read(&mut self) -> u8 {
        let a = self.st[self.p - self.k];
        // check keep mode bit, on keep mode, global pointer doesn't change but keep mode relative pointer does
        if !self.keep_mode {
            self.p -= 1;
        } else {
            self.k += 1;
        }
        a
    }

    pub fn read_short(&mut self) -> u16 {
        let b = self.read() as u16;
        let a = self.read() as u16;
        (a << 8) | b
    }

    pub fn write(&mut self, data: u8) {
        self.st[self.p] = data;
        self.p += 1;
    }

    pub fn write_short(&mut self, data: u16) {
        let a = (data / 256) as u8;
        let b = (data % 256) as u8;
        self.write(a);
        self.write(b);
    }
}
