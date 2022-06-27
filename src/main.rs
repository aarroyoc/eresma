use std::io::prelude::*;
use std::fs::File;

use num_enum::FromPrimitive;

// https://wiki.xxiivv.com/site/uxntal_reference.html
#[repr(u8)]
#[derive(FromPrimitive)]
enum Instruction {
    #[num_enum(default)]
    BRK = 0x00,
    INC = 0x01,
    POP = 0x02,
    NIP = 0x03,
    SWP = 0x04,
    ROT = 0x05,
    DUP = 0x06,
    OVR = 0x07,
    EQU = 0x08,
    NEQ = 0x09,
    GTH = 0x0a,
    LTH = 0x0b,
    JMP = 0x0c,
    JCN = 0x0d,
    JSR = 0x0e,
    STH = 0x0f,
    LDZ = 0x10,
    STZ = 0x11,
    LDR = 0x12,
    STR = 0x13,
    DEI = 0x16,
    DEO = 0x17,
    ADD = 0x18,
    SUB = 0x19,
    MUL = 0x1a,
    AND = 0x1c,
    ORA = 0x1d,
    EOR = 0x1e,
    SFT = 0x1f,
    INCr = 0x41,
    POPr = 0x42,
    NIPr = 0x43,
    SWPr = 0x44,
    ROTr = 0x45,
    DUPr = 0x46,
    OVRr = 0x47,
    EQUr = 0x48,
    NEQr = 0x49,
    GTHr = 0x4a,
    LTHr = 0x4b,
    LIT = 0x80,
    INCk = 0x81,
    POPk = 0x82,
    NIPk = 0x83,
    SWPk = 0x84,
    ROTk = 0x85,
    DUPk = 0x86,
    OVRk = 0x87,
    EQUk = 0x88,
    NEQk = 0x89,
    GTHk = 0x8a,
    LTHk = 0x8b,
    JMPk = 0x8c,
    JCNk = 0x8d,
    JSRk = 0x8e,
    STHk = 0x8f,
    ADDk = 0x98,
    SUBk = 0x99,
    MULk = 0x9a,
    ANDk = 0x9c,
    ORAk = 0x9d,
    EORk = 0x9e,
    SFTk = 0x9f,
    LIT2 = 0xa0,
    LITr = 0xc0,
    INCkr = 0xc1,
    POPkr = 0xc2,
    NIPkr = 0xc3,
    SWPkr = 0xc4,
    ROTkr = 0xc5,
    DUPkr = 0xc6,
    OVRkr = 0xc7,
    EQUkr = 0xc8,
    NEQkr = 0xc9,
    GTHkr = 0xca,
    LTHkr = 0xcb,
}

#[repr(u8)]
#[derive(FromPrimitive)]
enum Device {
    #[num_enum(default)]
    ConsoleWrite = 0x18
}

fn main() {
    match load_file() {
	Ok(code) => {execute(code);},
	Err(msg) => eprintln!("{}", msg)
    }
}

fn load_file() -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open("roms/hello.rom")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[derive(Clone)]
struct Stack {
    st: Vec<u8>, // Stack
    p: usize, // Pointer of the stack
    k: usize, // Keep Mode relative pointer
    keep_mode: bool,
    short_mode: bool,
}

impl Stack {
    fn new() -> Self {
	Stack {
	    st: vec![0; 256],
	    p : 0x00,
	    k: 1,
	    keep_mode: false,
	    short_mode: false,
	}
    }

    fn set_current_opcode(&mut self, opcode: u8) {
	self.k = 1; // reset keep mode relative pointer
	if opcode < 0x80 {
	    self.keep_mode = false;
	} else {
	    self.keep_mode = true;
	}
    }

    fn read(&mut self) -> u8 {
	let a = self.st[self.p-self.k];
	// check keep mode bit, on keep mode, global pointer doesn't change but keep mode relative pointer does
	if !self.keep_mode {
	    self.p -= 1;
	} else {
	    self.k += 1;
	}
	a
    }

    fn write(&mut self, data: u8) {
	self.st[self.p] = data;
	self.p += 1;
    }
}

struct MachineState {
    wst: Stack,
    rst: Stack,
    mem: Vec<u8>,
    pc: u16,
}

fn is_return_mode(opcode: u8) -> bool {
    (opcode > 0x40 && opcode < 0x80) || opcode >= 0xc0
}

fn execute(code: Vec<u8>) -> MachineState {
    let mut real_wst = Stack::new();
    let mut real_rst = Stack::new();
    let mut mem: Vec<u8> = vec![0; 65536];
    mem[0x0100..0x0100+code.len()].copy_from_slice(&code);
    let mut pc = 0x0100;
    loop {
	let (wst, rst) = if is_return_mode(mem[pc]) {
	    (&mut real_rst, &mut real_wst)
	} else {
	    (&mut real_wst, &mut real_rst)
	};

	wst.set_current_opcode(mem[pc]);
	match Instruction::from(mem[pc]) {
	    Instruction::BRK => {
		return MachineState {
		    wst: real_wst.clone(),
		    rst: real_rst.clone(),
		    mem,
		    pc: pc as u16,
		};
	    },
	    Instruction::LIT | Instruction::LITr => {
		wst.write(mem[pc+1]);
		pc += 2;
	    },
	    Instruction::LIT2 => {
		wst.write(mem[pc+1]);
		wst.write(mem[pc+2]);
		pc += 3;
	    },
	    Instruction::INC | Instruction::INCk | Instruction::INCr | Instruction::INCkr => {
		let a = wst.read();
		wst.write(a + 1);
		pc += 1;
	    },
	    Instruction::POP | Instruction::POPk | Instruction::POPr | Instruction::POPkr => {
		wst.read();
		pc += 1;
	    },
	    Instruction::NIP | Instruction::NIPk | Instruction::NIPr | Instruction::NIPkr => {
		let b = wst.read();
		let _ = wst.read();
		wst.write(b);
		pc += 1;
	    },
	    Instruction::SWP | Instruction::SWPk | Instruction::SWPr | Instruction::SWPkr => {
		let b = wst.read();
		let a = wst.read();
		wst.write(b);
		wst.write(a);
		pc += 1;
	    },
	    Instruction::ROT | Instruction::ROTk | Instruction::ROTr | Instruction::ROTkr => {
		let c = wst.read();
		let b = wst.read();
		let a = wst.read();
		wst.write(b);
		wst.write(c);
		wst.write(a);
		pc += 1;
	    },
	    Instruction::DUP | Instruction::DUPk | Instruction::DUPr | Instruction::DUPkr => {
		let a = wst.read();
		wst.write(a);
		wst.write(a);
		pc += 1;
	    },
	    Instruction::OVR | Instruction::OVRk | Instruction::OVRr | Instruction::OVRkr => {
		let b = wst.read();
		let a = wst.read();
		wst.write(a);
		wst.write(b);
		wst.write(a);
		pc += 1;
	    }
	    Instruction::EQU | Instruction::EQUk | Instruction::EQUr | Instruction::EQUkr => {
		let b = wst.read();
		let a = wst.read();
		let c = if a == b {
		    0x01
		} else {
		    0x00
		};
		wst.write(c);
		pc += 1;
	    }
	    Instruction::NEQ | Instruction::NEQk | Instruction::NEQr | Instruction::NEQkr => {
		let b = wst.read();
		let a = wst.read();
		let c = if a == b {
		    0x00
		} else {
		    0x01
		};
		wst.write(c);
		pc += 1;
	    }
	    Instruction::GTH | Instruction::GTHk | Instruction::GTHr | Instruction::GTHkr => {
		let b = wst.read();
		let a = wst.read();
		let c = if a < b {
		    0x00
		} else {
		    0x01
		};
		wst.write(c);
		pc += 1;
	    }
	    Instruction::LTH | Instruction::LTHk | Instruction::LTHr | Instruction::LTHkr => {
		let b = wst.read();
		let a = wst.read();
		let c = if a > b {
		    0x01
		} else {
		    0x00
		};
		wst.write(c);
		pc += 1;
	    }
	    Instruction::JMP | Instruction::JMPk => {
		let addr = wst.read();
		pc = (pc as i16 + addr as i16) as usize;
	    }
	    Instruction::JCN | Instruction::JCNk => {
		let addr = wst.read();
		let cond = wst.read();
		pc = if cond  == 0x00 {
		    pc + 0x01
		} else {
		    (pc as i16 + addr as i16) as usize
		};
	    }
	    Instruction::JSR | Instruction::JSRk => {
		let addr = wst.read();
		rst.write((pc - 0x0100) as u8);
		pc = (pc as i16 + addr as i16) as usize;
	    }
	    Instruction::STH | Instruction::STHk => {
		let a = wst.read();
		rst.write(a);
		pc += 1;
	    }
	    Instruction::LDZ => {
		let addr = wst.read();
		let val = mem[addr as usize];
		wst.write(val);
		pc += 1;
	    }
	    Instruction::STZ => {
		let addr = wst.read();
		let val = wst.read();
		mem[addr as usize] = val;
		pc += 1;
	    }
	    Instruction::LDR => {
		let addr = wst.read() as i16;
		let value = mem[((pc as i16) + addr) as usize];
		wst.write(value);
		pc += 1;
	    }
	    Instruction::STR => {
		let addr = wst.read() as i16;
		let val = wst.read();
		mem[((pc as i16) + addr) as usize] = val;
		pc += 1;
	    }
	    Instruction::DEI => {
		let device = wst.read();
		let val = device_read(Device::from(device));
		wst.write(val);
		pc += 1;
	    }
	    Instruction::DEO => {
		let device = wst.read();
		let val = wst.read();
		device_write(val, Device::from(device));
		pc += 1;
	    },
	    Instruction::ADD | Instruction::ADDk => {
		let b = wst.read();
		let a = wst.read();
		let c = a+b;
		wst.write(c);
		pc += 1;
	    },
	    Instruction::SUB | Instruction::SUBk => {
		let b = wst.read();
		let a = wst.read();
		let c = a-b;
		wst.write(c);
		pc += 1;
	    },
	    Instruction::MUL | Instruction::MULk => {
		let b = wst.read();
		let a = wst.read();
		let c = a*b;
		wst.write(c);
		pc += 1;
	    }
	    Instruction::AND | Instruction::ANDk => {
		let b = wst.read();
		let a = wst.read();
		let c = a & b;
		wst.write(c);
		pc += 1;
	    }
	    Instruction::ORA | Instruction::ORAk => {
		let b = wst.read();
		let a = wst.read();
		let c = a | b;
		wst.write(c);
		pc += 1;
	    }
	    Instruction::EOR | Instruction::EORk => {
		let b = wst.read();
		let a = wst.read();
		let c = a ^ b;
		wst.write(c);
		pc += 1;
	    }
	    Instruction::SFT | Instruction::SFTk => {
		let shift = wst.read();
		let a = wst.read();
		let left = shift / 16;
		let right = shift % 16;
		let c = (a >> right) << left;
		wst.write(c);
		pc += 1;
	    }
	    /*Instruction::ADD2 => {
		let a: u16 = wst[wst_pointer-4] as u16 * 256 + wst[wst_pointer-3] as u16;
		let b: u16 = wst[wst_pointer-2] as u16 * 256 + wst[wst_pointer-1] as u16;
		let sum = a + b;
		wst[wst_pointer-4] = (sum / 256) as u8;
		wst[wst_pointer-3] = (sum % 256) as u8;
		wst_pointer -= 2;
		pc += 1;
	    }*/
	}
    }
}

fn device_read(device: Device) -> u8 {
    unimplemented!();
}

fn device_write(val: u8, device: Device) {
    match device {
	Device::ConsoleWrite => {
	    print!("{}", val as char);
	}
    }
}

#[test]
fn lit() {
    let code = vec![0x80, 0x05];
    let mut wst = vec![0; 256];
    wst[0] = 0x05;
    let mut memory = vec![0; 65536];
    memory[0x0100] = 0x80;
    memory[0x0101] = 0x05;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(memory, state.mem);
}

#[test]
fn lit2() {
    let code = vec![0x80, 0x05];
    let mut wst = vec![0; 256];
    wst[0] = 0x05;
    let mut memory = vec![0; 65536];
    memory[0x0100] = 0x80;
    memory[0x0101] = 0x05;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(memory, state.mem);
}

#[test]
fn inc() {
    let code = vec![0x80, 0x05, 0x01];
    let mut wst = vec![0; 256];
    wst[0] = 0x06;
    let mut memory = vec![0; 65536];
    memory[0x0100] = 0x80;
    memory[0x0101] = 0x05;
    memory[0x0102] = 0x01;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(memory, state.mem);
}

#[test]
fn inc_keep() {
    let code = vec![0x80, 0x05, 0x81];
    let mut wst = vec![0; 256];
    wst[0] = 0x05;
    wst[1] = 0x06;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(2, state.wst.p);
}

#[test]
fn inc_return() {
    let code = vec![0xc0, 0x05, 0x41];
    let wst = vec![0; 256];
    let mut rst = vec![0; 256];
    rst[0] = 0x06;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(0, state.wst.p);
    assert_eq!(rst, state.rst.st);
    assert_eq!(1, state.rst.p);
}

#[test]
fn inc_keep_return() {
    let code = vec![0xc0, 0x05, 0xc1];
    let wst = vec![0; 256];
    let mut rst = vec![0; 256];
    rst[0] = 0x05; 
    rst[1] = 0x06;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(0, state.wst.p);
    assert_eq!(rst, state.rst.st);
    assert_eq!(2, state.rst.p);
}

#[test]
fn pop() {
    let code = vec![0xa0, 0x12, 0x34, 0x02];
    let mut wst = vec![0; 256];
    wst[0] = 0x12;
    wst[1] = 0x34; 
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn nip() {
    let code = vec![0xa0, 0x12, 0x34, 0x03];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn swp() {
    let code = vec![0xa0, 0x12, 0x34, 0x04];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(2, state.wst.p);
}

#[test]
fn add() {
    let code = vec![0xa0, 0x12, 0x34, 0x18];
    let mut wst = vec![0; 256];
    wst[0] = 0x12 + 0x34;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn sub() {
    let code = vec![0xa0, 0x34, 0x12, 0x19];
    let mut wst = vec![0; 256];
    wst[0] = 0x34 - 0x12;
    wst[1] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn add2() {
    let code = vec![0xa0, 0x00, 0x04, 0xa0, 0x00, 0x08, 0x38];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x0c;
    wst[2] = 0x00;
    wst[3] = 0x08;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(2, state.wst.p);
}

#[test]
fn rot() {
    let code = vec![0xa0, 0x12, 0x34, 0x80, 0x56, 0x05];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x56;
    wst[2] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn dup() {
    let code = vec![0xa0, 0x12, 0x34, 0x06];
    let mut wst = vec![0; 256];
    wst[0] = 0x12;
    wst[1] = 0x34;
    wst[2] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn ovr() {
    let code = vec![0xa0, 0x12, 0x34, 0x07];
    let mut wst = vec![0; 256];
    wst[0] = 0x12;
    wst[1] = 0x34;
    wst[2] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn equ() {
    let code = vec![0xa0, 0x12, 0x12, 0x08];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn equ_() {
    let code = vec![0xa0, 0x12, 0x13, 0x08];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x13;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn neq() {
    let code = vec![0xa0, 0x12, 0x34, 0x09];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn neq_() {
    let code = vec![0xa0, 0x12, 0x12, 0x09];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn gth() {
    let code = vec![0xa0, 0x12, 0x34, 0x0a];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn gth_() {
    let code = vec![0xa0, 0x34, 0x12, 0x0a];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn lth() {
    let code = vec![0xa0, 0x01, 0x01, 0x0b];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x01;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn lth_() {
    let code = vec![0xa0, 0x01, 0x00, 0x0b];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x00;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn jmp() {
    let code = vec![0xa0, 0x55, 0x34, 0x0c];
    let mut wst = vec![0; 256];
    wst[0] = 0x55;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
    assert_eq!(0x0100 + 0x02 + 0x34 + 0x01, state.pc);
}

#[test]
fn jcn() {
    let code = vec![0xa0, 0x01, 0x34, 0x0d];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(0, state.wst.p);
    assert_eq!(0x0100 + 0x02 + 0x34 + 0x01, state.pc);
}

#[test]
fn jsr() {
    let code = vec![0xa0, 0x12, 0x34, 0x0e];
    let mut wst = vec![0; 256];
    wst[0] = 0x12;
    wst[1] = 0x34;
    let mut rst = vec![0; 256];
    rst[0] = 0x03;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
    assert_eq!(0x0100 + 0x02 + 0x34 + 0x01, state.pc);
    assert_eq!(rst, state.rst.st);
}

#[test]
fn sth() {
    let code = vec![0xa0, 0x12, 0x34, 0x0f];
    let mut wst = vec![0; 256];
    wst[0] = 0x12;
    wst[1] = 0x34;
    let mut rst = vec![0; 256];
    rst[0] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
    assert_eq!(rst, state.rst.st);
}

#[test]
fn ldz_and_stz() {
    let code = vec![0xa0, 0x50, 0x00, 0x11, 0x80, 0x00, 0x10];
    let mut wst = vec![0; 256];
    wst[0] = 0x50;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
}

#[test]
fn ldr_and_str() {
    let code = vec![0xa0, 0x50, 0x10, 0x13, 0x80, 0x07, 0x12];
    let mut wst = vec![0; 256];
    wst[0] = 0x50;
    let state = execute(code);
    dbg!(state.mem[0x0113]);
    assert_eq!(wst, state.wst.st);
}

#[test]
fn mul_keep() {
    let code = vec![0xa0, 0x50, 0x02, 0x9a];
    let mut wst = vec![0; 256];
    wst[0] = 0x50;
    wst[1] = 0x02;
    wst[2] = 0xa0;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn and() {
    let code = vec![0xa0, 0xf0, 0x0f, 0x1c];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x0f;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn ora_keep() {
    let code = vec![0xa0, 0xf0, 0xff, 0x9d];
    let mut wst = vec![0; 256];
    wst[0] = 0xf0;
    wst[1] = 0xff;
    wst[2] = 0xff;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn eor_keep() {
    let code = vec![0xa0, 0xf0, 0xff, 0x9e];
    let mut wst = vec![0; 256];
    wst[0] = 0xf0;
    wst[1] = 0xff;
    wst[2] = 0x0f;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn sft() {
    let code = vec![0xa0, 0x34, 0x10, 0x1f];
    let mut wst = vec![0; 256];
    wst[0] = 0x68;
    wst[1] = 0x10;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn sft_keep() {
    let code = vec![0xa0, 0x34, 0x33, 0x9f];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x33;
    wst[2] = 0x30;
    let state = execute(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}
