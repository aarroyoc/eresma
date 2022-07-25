#![allow(clippy::upper_case_acronyms)]
use std::fs::File;
use std::io::prelude::*;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, *};
use ggez::{Context, GameResult};
use num_enum::FromPrimitive;

mod stack;

use stack::Stack;

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
    INC2 = 0x21,
    DEI2 = 0x36,
    DEO2 = 0x37,
    ADD2 = 0x38,
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
    LIT2r = 0xe0,
}

#[repr(u8)]
#[derive(FromPrimitive)]
enum Device {
    #[num_enum(default)]
    SystemRedHigh = 0x08,
    SystemRedLow = 0x09,
    SystemGreenHigh = 0x0a,
    SystemGreenLow = 0x0b,
    SystemBlueHigh = 0x0c,
    SystemBlueLow = 0x0d,
    ConsoleWrite = 0x18,
    ScreenVector = 0x20,
    ScreenWidth = 0x22,
    ScreenHeight = 0x24,
    ScreenAuto = 0x26,
    ScreenXHigh = 0x28,
    ScreenXLow = 0x29,
    ScreenYHigh = 0x2a,
    ScreenYLow = 0x2b,
    ScreenMemAddress = 0x2c,
    ScreenPixel = 0x2e,
    ScreenSprite = 0x2f,
}

const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 320;
const SCREEN_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

#[derive(Clone)]
struct Devices {
    system: [u8; 16],
    screen: [u8; 16],
    screen_buffer_bg: Vec<u8>,
    screen_buffer_fg: Vec<u8>,
}

impl Default for Devices {
    fn default() -> Self {
        Devices {
            system: [0; 16],
            screen: [0; 16],
            screen_buffer_bg: vec![0; SCREEN_SIZE],
            screen_buffer_fg: vec![0; SCREEN_SIZE],
        }
    }
}

impl Devices {
    fn write(&mut self, val: u8, device: u8) {
        match Device::from(device) {
            Device::SystemRedHigh => {
                self.system[8] = val;
            }
            Device::SystemRedLow => {
                self.system[9] = val;
            }
            Device::SystemGreenHigh => {
                self.system[10] = val;
            }
            Device::SystemGreenLow => {
                self.system[11] = val;
            }
            Device::SystemBlueHigh => {
                self.system[12] = val;
            }
            Device::SystemBlueLow => {
                self.system[13] = val;
            }
            Device::ConsoleWrite => {
                print!("{}", val as char);
            }
            Device::ScreenXHigh => {
                self.screen[7] = val;
            }
            Device::ScreenXLow => {
                self.screen[8] = val;
            }
            Device::ScreenYHigh => {
                self.screen[9] = val;
            }
            Device::ScreenYLow => {
                self.screen[10] = val;
            }
            Device::ScreenPixel => {
                let x: u16 = (self.screen[7] as u16) * 256 + self.screen[8] as u16;
                let y: u16 = (self.screen[9] as u16) * 256 + self.screen[10] as u16;
                let color0 = [
                    (self.system[8] >> 4) | (self.system[8] >> 4) << 4,
                    (self.system[10] >> 4) | (self.system[10] >> 4) << 4,
                    (self.system[12] >> 4) | (self.system[12] >> 4) << 4,
                    0xff,
                ];
                let color1 = [
                    (self.system[8] << 4) | (self.system[8] << 4) >> 4,
                    (self.system[10] << 4) | (self.system[10] << 4) >> 4,
                    (self.system[12] << 4) | (self.system[12] << 4) >> 4,
                    0xff,
                ];
                let color2 = [
                    (self.system[9] >> 4) | (self.system[9] >> 4) << 4,
                    (self.system[11] >> 4) | (self.system[11] >> 4) << 4,
                    (self.system[13] >> 4) | (self.system[13] >> 4) << 4,
                    0xff,
                ];
                let color3 = [
                    (self.system[9] << 4) | (self.system[9] << 4) >> 4,
                    (self.system[11] << 4) | (self.system[11] << 4) >> 4,
                    (self.system[13] << 4) | (self.system[13] << 4) >> 4,
                    0xff,
                ];
                match val {
                    0x00 => self.draw_screen_bg(x, y, color0),
                    0x01 => self.draw_screen_bg(x, y, color1),
                    0x02 => self.draw_screen_bg(x, y, color2),
                    0x03 => self.draw_screen_bg(x, y, color3),
                    0x40 => self.draw_screen_fg(x, y, color0),
                    0x41 => self.draw_screen_fg(x, y, color1),
                    0x42 => self.draw_screen_fg(x, y, color2),
                    0x43 => self.draw_screen_fg(x, y, color3),
                    _ => {}
                }
            }
            _ => todo!(),
        }
    }

    fn draw_screen_bg(&mut self, x: u16, y: u16, color: [u8; 4]) {
        let base: usize = ((x as usize) + (y as usize * SCREEN_HEIGHT)) * 4;
        self.screen_buffer_bg[base] = color[0];
        self.screen_buffer_bg[base + 1] = color[1];
        self.screen_buffer_bg[base + 2] = color[2];
        self.screen_buffer_bg[base + 3] = color[3];
    }

    fn draw_screen_fg(&mut self, x: u16, y: u16, color: [u8; 4]) {
        let base: usize = ((x as usize) + (y as usize * SCREEN_HEIGHT)) * 4;
        self.screen_buffer_fg[base] = color[0];
        self.screen_buffer_fg[base + 1] = color[1];
        self.screen_buffer_fg[base + 2] = color[2];
        self.screen_buffer_fg[base + 3] = color[3];
    }

    fn write_short(&mut self, val: u16, device: u8) {
        let next_device = device + 1;
        self.write((val / 256) as u8, device);
        self.write((val % 256) as u8, next_device);
    }

    fn read(&self, device: u8) -> u8 {
        match Device::from(device) {
            Device::ScreenXHigh => self.screen[7],
            Device::ScreenXLow => self.screen[8],
            Device::ScreenYHigh => self.screen[9],
            Device::ScreenYLow => self.screen[10],
            _ => todo!(),
        }
    }

    fn read_short(&self, device: u8) -> u16 {
        let high = self.read(device) as u16;
        let low = self.read(device + 1) as u16;
        high * 256 + low
    }
}

struct MachineState {
    wst: Stack,
    rst: Stack,
    mem: Vec<u8>,
    pc: u16,
    devices: Devices,
}

impl MachineState {
    fn from_code(code: Vec<u8>) -> Self {
        let mut mem: Vec<u8> = vec![0; 65536];
        mem[0x0100..0x0100 + code.len()].copy_from_slice(&code);
        MachineState {
            wst: Stack::new(),
            rst: Stack::new(),
            mem,
            pc: 0x0100,
            devices: Devices::default(),
        }
    }

    fn from_file(file: &str) -> GameResult<MachineState> {
        match MachineState::load_file(file) {
            Ok(code) => Ok(execute(code)),
            Err(_msg) => Err(ggez::GameError::FilesystemError(
                "Can't load file".to_string(),
            )),
        }
    }
    fn load_file(file: &str) -> Result<MachineState, std::io::Error> {
        let mut file = File::open(file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut mem: Vec<u8> = vec![0; 65536];
        mem[0x0100..0x0100 + buffer.len()].copy_from_slice(&buffer);
        Ok(MachineState {
            wst: Stack::new(),
            rst: Stack::new(),
            mem,
            pc: 0x0100,
            devices: Devices::default(),
        })
    }
}

impl event::EventHandler<ggez::GameError> for MachineState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let image_bg = Image::from_rgba8(
            ctx,
            SCREEN_WIDTH as u16,
            SCREEN_HEIGHT as u16,
            &self.devices.screen_buffer_bg,
        )?;
        image_bg.draw(ctx, DrawParam::new())?;

        let image_fg = Image::from_rgba8(
            ctx,
            SCREEN_WIDTH as u16,
            SCREEN_HEIGHT as u16,
            &self.devices.screen_buffer_fg,
        )?;
        image_fg.draw(ctx, DrawParam::new())?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("eresma", "aarroyoc");
    let cb = cb.window_setup(WindowSetup {
        title: "Eresma - UXN/Varvara Computer".to_string(),
        ..WindowSetup::default()
    });
    let cb = cb.window_mode(WindowMode {
        width: 512.0,
        height: 320.0,
        ..WindowMode::default()
    });
    let (mut ctx, event_loop) = cb.build()?;
    graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);
    let state = MachineState::from_file("roms/hello-pixels.rom")?;
    event::run(ctx, event_loop, state)
}

fn is_return_mode(opcode: u8) -> bool {
    (opcode > 0x40 && opcode < 0x80) || opcode >= 0xc0
}

#[allow(clippy::redundant_clone)]
fn execute(state: MachineState) -> MachineState {
    let mut real_wst = state.wst.clone();
    let mut real_rst = state.rst.clone();
    let mut mem = state.mem.clone();
    let mut pc = state.pc as usize;
    let mut devices = state.devices.clone();
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
                    devices: devices.clone(),
                };
            }
            Instruction::LIT | Instruction::LITr => {
                wst.write(mem[pc + 1]);
                pc += 2;
            }
            Instruction::LIT2 | Instruction::LIT2r => {
                wst.write(mem[pc + 1]);
                wst.write(mem[pc + 2]);
                pc += 3;
            }
            Instruction::INC | Instruction::INCk | Instruction::INCr | Instruction::INCkr => {
                let a = wst.read();
                wst.write(a + 1);
                pc += 1;
            }
            Instruction::POP | Instruction::POPk | Instruction::POPr | Instruction::POPkr => {
                wst.read();
                pc += 1;
            }
            Instruction::NIP | Instruction::NIPk | Instruction::NIPr | Instruction::NIPkr => {
                let b = wst.read();
                let _ = wst.read();
                wst.write(b);
                pc += 1;
            }
            Instruction::SWP | Instruction::SWPk | Instruction::SWPr | Instruction::SWPkr => {
                let b = wst.read();
                let a = wst.read();
                wst.write(b);
                wst.write(a);
                pc += 1;
            }
            Instruction::ROT | Instruction::ROTk | Instruction::ROTr | Instruction::ROTkr => {
                let c = wst.read();
                let b = wst.read();
                let a = wst.read();
                wst.write(b);
                wst.write(c);
                wst.write(a);
                pc += 1;
            }
            Instruction::DUP | Instruction::DUPk | Instruction::DUPr | Instruction::DUPkr => {
                let a = wst.read();
                wst.write(a);
                wst.write(a);
                pc += 1;
            }
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
                let c = if a == b { 0x01 } else { 0x00 };
                wst.write(c);
                pc += 1;
            }
            Instruction::NEQ | Instruction::NEQk | Instruction::NEQr | Instruction::NEQkr => {
                let b = wst.read();
                let a = wst.read();
                let c = if a == b { 0x00 } else { 0x01 };
                wst.write(c);
                pc += 1;
            }
            Instruction::GTH | Instruction::GTHk | Instruction::GTHr | Instruction::GTHkr => {
                let b = wst.read();
                let a = wst.read();
                let c = if a < b { 0x00 } else { 0x01 };
                wst.write(c);
                pc += 1;
            }
            Instruction::LTH | Instruction::LTHk | Instruction::LTHr | Instruction::LTHkr => {
                let b = wst.read();
                let a = wst.read();
                let c = if a > b { 0x01 } else { 0x00 };
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
                pc = if cond == 0x00 {
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
                let val = devices.read(device);
                wst.write(val);
                pc += 1;
            }
            Instruction::DEO => {
                let device = wst.read();
                let val = wst.read();
                devices.write(val, device);
                pc += 1;
            }
            Instruction::ADD | Instruction::ADDk => {
                let b = wst.read();
                let a = wst.read();
                let c = a + b;
                wst.write(c);
                pc += 1;
            }
            Instruction::SUB | Instruction::SUBk => {
                let b = wst.read();
                let a = wst.read();
                let c = a - b;
                wst.write(c);
                pc += 1;
            }
            Instruction::MUL | Instruction::MULk => {
                let b = wst.read();
                let a = wst.read();
                let c = a * b;
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
            Instruction::ADD2 => {
                let b = wst.read_short();
                let a = wst.read_short();
                let c = a + b;
                wst.write_short(c);
                pc += 1;
            }
            Instruction::INC2 => {
                let a = wst.read_short();
                wst.write_short(a + 1);
                pc += 1;
            }
            Instruction::DEI2 => {
                let device = wst.read();
                let val = devices.read_short(device);
                wst.write_short(val);
                pc += 1;
            }
            Instruction::DEO2 => {
                let device = wst.read();
                let val = wst.read_short();
                devices.write_short(val, device);
                pc += 1;
            }
        }
    }
}

#[allow(dead_code)]
fn execute_test(code: Vec<u8>) -> MachineState {
    let state = MachineState::from_code(code);
    execute(state)
}

#[test]
fn lit() {
    let code = vec![0x80, 0x05];
    let mut wst = vec![0; 256];
    wst[0] = 0x05;
    let mut memory = vec![0; 65536];
    memory[0x0100] = 0x80;
    memory[0x0101] = 0x05;
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(memory, state.mem);
}

#[test]
fn inc_keep() {
    let code = vec![0x80, 0x05, 0x81];
    let mut wst = vec![0; 256];
    wst[0] = 0x05;
    wst[1] = 0x06;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(2, state.wst.p);
}

#[test]
fn inc_return() {
    let code = vec![0xc0, 0x05, 0x41];
    let wst = vec![0; 256];
    let mut rst = vec![0; 256];
    rst[0] = 0x06;
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn nip() {
    let code = vec![0xa0, 0x12, 0x34, 0x03];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x34;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn swp() {
    let code = vec![0xa0, 0x12, 0x34, 0x04];
    let mut wst = vec![0; 256];
    wst[0] = 0x34;
    wst[1] = 0x12;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(2, state.wst.p);
}

#[test]
fn add() {
    let code = vec![0xa0, 0x12, 0x34, 0x18];
    let mut wst = vec![0; 256];
    wst[0] = 0x12 + 0x34;
    wst[1] = 0x34;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn sub() {
    let code = vec![0xa0, 0x34, 0x12, 0x19];
    let mut wst = vec![0; 256];
    wst[0] = 0x34 - 0x12;
    wst[1] = 0x12;
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn equ() {
    let code = vec![0xa0, 0x12, 0x12, 0x08];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x12;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn equ_() {
    let code = vec![0xa0, 0x12, 0x13, 0x08];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x13;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn neq() {
    let code = vec![0xa0, 0x12, 0x34, 0x09];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x34;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn neq_() {
    let code = vec![0xa0, 0x12, 0x12, 0x09];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x12;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn gth() {
    let code = vec![0xa0, 0x12, 0x34, 0x0a];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x34;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn gth_() {
    let code = vec![0xa0, 0x34, 0x12, 0x0a];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x12;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn lth() {
    let code = vec![0xa0, 0x01, 0x01, 0x0b];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x01;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn lth_() {
    let code = vec![0xa0, 0x01, 0x00, 0x0b];
    let mut wst = vec![0; 256];
    wst[0] = 0x01;
    wst[1] = 0x00;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
}

#[test]
fn jmp() {
    let code = vec![0xa0, 0x55, 0x34, 0x0c];
    let mut wst = vec![0; 256];
    wst[0] = 0x55;
    wst[1] = 0x34;
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(1, state.wst.p);
    assert_eq!(rst, state.rst.st);
}

#[test]
fn ldz_and_stz() {
    let code = vec![0xa0, 0x50, 0x00, 0x11, 0x80, 0x00, 0x10];
    let mut wst = vec![0; 256];
    wst[0] = 0x50;
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
}

#[test]
fn ldr_and_str() {
    let code = vec![0xa0, 0x50, 0x10, 0x13, 0x80, 0x07, 0x12];
    let mut wst = vec![0; 256];
    wst[0] = 0x50;
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn and() {
    let code = vec![0xa0, 0xf0, 0x0f, 0x1c];
    let mut wst = vec![0; 256];
    wst[0] = 0x00;
    wst[1] = 0x0f;
    let state = execute_test(code);
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
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}

#[test]
fn sft() {
    let code = vec![0xa0, 0x34, 0x10, 0x1f];
    let mut wst = vec![0; 256];
    wst[0] = 0x68;
    wst[1] = 0x10;
    let state = execute_test(code);
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
    let state = execute_test(code);
    assert_eq!(wst, state.wst.st);
    assert_eq!(3, state.wst.p);
}
