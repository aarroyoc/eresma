use std::io::BufReader;
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
    LIT = 0x80,
    LIT2 = 0xa0,
}

fn main() {
    match load_file() {
	Ok(code) => {execute(code);},
	Err(msg) => eprintln!("{}", msg)
    }
}

fn load_file() -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open("literals.rom")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// fn parse_code() -> Result<Vec<Instruction>, String> {
//     let file = File::open("literals.rom").unwrap();
//     let reader = BufReader::new(file);
//     let mut bytes = reader.bytes();
//     let mut instructions = Vec::new();
//     while let Some(opcode_byte) = bytes.next() {
// 	match opcode_byte {
// 	    Ok(opcode) => match opcode {
// 		0x00 => instructions.push(Instruction::BRK),
// 		0x80 => {
// 		    instructions.push(Instruction::LIT);
// 		    instructions.push(Instruction::BYTE(bytes.next_byte()?));
// 		},
// 		0xa0 => {
// 		    instructions.push(Instruction::LIT2);
// 		    instructions.push(Instruction::BYTE(bytes.next_byte()?));
// 		    instructions.push(Instruction::BYTE(bytes.next_byte()?));
// 		}
// 		_ => ()
// 	    },
// 	    Err(_) => return Err("Can't read bytes".to_string())
// 	};	
//     }
//     return Ok(instructions);
// }

trait NextByte {
    fn next_byte(&mut self) -> Result<u8, String>;
}

impl NextByte for std::io::Bytes<BufReader<File>> {
    fn next_byte(&mut self) -> Result<u8, String> {
        match self.next() {
	    Some(byte) => match byte {
	        Ok(byte) => Ok(byte),
	        Err(_) => Err("Can't read bytes".to_string())
	    },
	    None => Err("No more bytes".to_string())
        }	
    }
}

struct MachineState {
    wst: Vec<u8>,
    wst_pointer: u8,
    rst: Vec<u8>,
    mem: Vec<u8>,
    pc: u16,
}

fn execute(code: Vec<u8>) -> MachineState {
    let mut wst: Vec<u8> = vec![0; 256];
    let mut rst: Vec<u8> = vec![0; 256];
    let mut mem: Vec<u8> = vec![0; 65536];
    mem[0x0100..0x0100+code.len()].copy_from_slice(&code);
    let mut pc = 0x0100;
    let mut wst_pointer = 0x00;
    loop {
	match Instruction::from(mem[pc]) {
	    Instruction::BRK => {
		return MachineState {
		    wst,
		    wst_pointer: wst_pointer as u8,
		    rst,
		    mem,
		    pc: pc as u16,
		};
	    },
	    Instruction::LIT => {
		wst[wst_pointer] = mem[pc+1];
		wst_pointer += 1;
		pc += 2;
	    },
	    Instruction::LIT2 => {
		wst[wst_pointer] = mem[pc+1];
		wst[wst_pointer + 1] = mem[pc+2];
		wst_pointer += 2;
		pc += 3;
	    },
	    Instruction::INC => {
		wst[wst_pointer-1] += 1;
		pc += 1;
	    },
	    Instruction::POP => {
		wst_pointer -= 1;
		pc += 1;
	    },
	    Instruction::NIP => {
		wst[wst_pointer-2] = wst[wst_pointer-1];
		wst_pointer -= 1;
		pc += 1;
	    },
	    Instruction::SWP => {
		let swp = wst[wst_pointer-2];
		wst[wst_pointer-2] = wst[wst_pointer-1];
		wst[wst_pointer-1] = swp;
		pc += 1;
	    }
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
    assert_eq!(wst, state.wst);
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
    assert_eq!(wst, state.wst);
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
    assert_eq!(wst, state.wst);
    assert_eq!(memory, state.mem);
}

#[test]
fn pop() {
    let code = vec![0xa0, 0x12, 0x34, 0x02];
    let mut wst = vec![0; 256];
    wst[0] = 0x12;
    wst[1] = 0x34; 
    let state = execute(code);
    assert_eq!(wst, state.wst);
    assert_eq!(1, state.wst_pointer);
}

#[test]
fn nip() {
    let code = vec![0xa0, 0x12, 0x34, 0x03];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x34;
    let state = execute(code);
    assert_eq!(wst, state.wst);
    assert_eq!(1, state.wst_pointer);
}

#[test]
fn swp() {
    let code = vec![0xa0, 0x12, 0x34, 0x04];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x12;
    let state = execute(code);
    assert_eq!(wst, state.wst);
    assert_eq!(2, state.wst_pointer);
}
