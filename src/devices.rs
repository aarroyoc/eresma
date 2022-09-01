use num_enum::FromPrimitive;

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
    ScreenVectorHigh = 0x20,
    ScreenVectorLow = 0x21,
    ScreenWidthHigh = 0x22,
    ScreenWidthLow = 0x23,
    ScreenHeightHigh = 0x24,
    ScreenHeightLow = 0x25,
    ScreenAuto = 0x26,
    ScreenXHigh = 0x28,
    ScreenXLow = 0x29,
    ScreenYHigh = 0x2a,
    ScreenYLow = 0x2b,
    ScreenAddressHigh = 0x2c,
    ScreenAddressLow = 0x2d,
    ScreenPixel = 0x2e,
    ScreenSprite = 0x2f,
    ControllerVectorHigh = 0x80,
    ControllerVectorLow = 0x81,
    ControllerButton = 0x82,
    ControllerKey = 0x83,
}

pub const SCREEN_WIDTH: usize = 512;
const SCREEN_WIDTH_HIGH: u8 = (SCREEN_WIDTH / 256) as u8;
const SCREEN_WIDTH_LOW: u8 = (SCREEN_WIDTH % 256) as u8;
pub const SCREEN_HEIGHT: usize = 312;
const SCREEN_HEIGHT_HIGH: u8 = (SCREEN_HEIGHT / 256) as u8;
const SCREEN_HEIGHT_LOW: u8 = (SCREEN_HEIGHT % 256) as u8;
const SCREEN_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

#[derive(Clone)]
pub struct Devices {
    system: [u8; 16],
    screen: [u8; 16],
    controller: [u8; 4],
    pub screen_buffer_bg: Vec<u8>,
    pub screen_buffer_fg: Vec<u8>,
}

impl Default for Devices {
    fn default() -> Self {
        Devices {
            system: [0; 16],
            screen: [0; 16],
	    controller: [0; 4],
            screen_buffer_bg: vec![0; SCREEN_SIZE],
            screen_buffer_fg: vec![0; SCREEN_SIZE],
        }
    }
}

impl Devices {
    pub fn write(&mut self, val: u8, device: u8, mem: &Vec<u8>) {
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
	    Device::ScreenVectorHigh => {
		self.screen[0] = val;
	    }
	    Device::ScreenVectorLow => {
		self.screen[1] = val;
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
            Device::ScreenAddressHigh => {
                self.screen[11] = val;
            }
            Device::ScreenAddressLow => {
                self.screen[12] = val;
            }
            Device::ScreenPixel => {
                let x: u16 = self.get_screen_x();
                let y: u16 = self.get_screen_y();
                let color0 = self.get_color0();
                let color1 = self.get_color1();
                let color2 = self.get_color2();
                let color3 = self.get_color3();

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
            Device::ScreenSprite => {
                let address: usize = (self.screen[11] as usize) * 256 + self.screen[12] as usize;
                if val > 127 {
                    self.draw_sprite_2bpp(address, mem, val);
                } else {
                    self.draw_sprite_1bpp(address, mem, val);
                }
            }
	    Device::ControllerVectorHigh => {
		self.controller[0] = val;
	    }
	    Device::ControllerVectorLow => {
		self.controller[1] = val;
	    }
            _ => todo!(),
        }
    }

    fn get_sprite_color(&self, val: u8) -> [Option<[u8; 4]>; 4] {
        let color0 = self.get_color0();
        let color1 = self.get_color1();
        let color2 = self.get_color2();
        let color3 = self.get_color3();
        match val & 0b00001111 {
            0x00 => [Some(color0), Some(color0), Some(color1), Some(color2)],
            0x01 => [Some(color0), Some(color1), Some(color2), Some(color3)],
            0x02 => [Some(color0), Some(color2), Some(color3), Some(color1)],
            0x03 => [Some(color0), Some(color3), Some(color1), Some(color2)],
            0x04 => [Some(color1), Some(color0), Some(color1), Some(color2)],
            0x05 => [None, Some(color1), Some(color2), Some(color3)],
            0x06 => [Some(color1), Some(color2), Some(color3), Some(color1)],
            0x07 => [Some(color1), Some(color3), Some(color1), Some(color2)],
            0x08 => [Some(color2), Some(color0), Some(color1), Some(color2)],
            0x09 => [Some(color2), Some(color1), Some(color2), Some(color3)],
            0x0a => [None, Some(color2), Some(color3), Some(color1)],
            0x0b => [Some(color2), Some(color3), Some(color1), Some(color2)],
            0x0c => [Some(color3), Some(color0), Some(color1), Some(color2)],
            0x0d => [Some(color3), Some(color1), Some(color2), Some(color3)],
            0x0e => [Some(color3), Some(color2), Some(color3), Some(color1)],
            0x0f => [None, Some(color3), Some(color1), Some(color2)],
            _ => unreachable!(),
        }
    }

    fn draw_sprite_1bpp(&mut self, address: usize, mem: &Vec<u8>, val: u8) {
        let x = self.get_screen_x();
        let y = self.get_screen_y();
        let sprite_colors = self.get_sprite_color(val);
        for i in 0..8 {
            let line = mem[address + i];
            let mut mask = 0b10000000;

            for j in 0..8 {
                let pixel = (line & mask) > 0;
                mask = mask >> 1;

                let i = i as u16;

                if val & 0b00001111 == 0 {
                    self.draw_screen_fg(x + j, y + i, [0, 0, 0, 0]);
                } else {
                    if pixel {
                        if let Some(color) = sprite_colors[1] {
                            self.draw_screen_fg(x + j, y + i, color);
                        }
                    } else {
                        if let Some(color) = sprite_colors[0] {
                            self.draw_screen_fg(x + j, y + i, color);
                        }
                    }
                }
            }
        }
    }

    fn draw_sprite_2bpp(&mut self, address: usize, mem: &Vec<u8>, val: u8) {
        let x = self.get_screen_x();
        let y = self.get_screen_y();
        let sprite_colors = self.get_sprite_color(val);
        for i in 0..8 {
            let line1 = mem[address + i];
            let line2 = mem[address + 8 + i];
            let mut mask = 0b10000000;

            for j in 0..8 {
                let pixel1 = (line1 & mask) > 0;
                let pixel2 = (line2 & mask) > 0;
                mask = mask >> 1;

                let i = i as u16;

                match (pixel1, pixel2) {
                    (false, false) => {
                        if let Some(color) = sprite_colors[0] {
                            self.draw_screen_fg(x + j, y + i, color);
                        }
                    }
                    (false, true) => {
                        if let Some(color) = sprite_colors[1] {
                            self.draw_screen_fg(x + j, y + i, color);
                        }
                    }
                    (true, false) => {
                        if let Some(color) = sprite_colors[2] {
                            self.draw_screen_fg(x + j, y + i, color);
                        }
                    }
                    (true, true) => {
                        if let Some(color) = sprite_colors[3] {
                            self.draw_screen_fg(x + j, y + i, color);
                        }
                    }
                }
            }
        }
    }

    pub fn get_button(&self) -> u8 {
	self.controller[2]
    }

    pub fn set_button(&mut self, button: u8) {
	self.controller[2] = button;
    }

    pub fn get_key(&self) -> u8 {
	self.controller[3]
    }

    pub fn set_key(&mut self, key: u8) {
	self.controller[3] = key;
    }

    pub fn get_controller_vector(&self) -> u16 {
	(self.controller[0] as u16) * 256 + self.controller[1] as u16
    }

    pub fn get_screen_vector(&self) -> u16 {
        (self.screen[0] as u16) * 256 + self.screen[1] as u16
    }

    fn get_screen_x(&self) -> u16 {
        (self.screen[7] as u16) * 256 + self.screen[8] as u16
    }

    fn get_screen_y(&self) -> u16 {
        (self.screen[9] as u16) * 256 + self.screen[10] as u16
    }

    fn get_color0(&self) -> [u8; 4] {
        [
            (self.system[8] >> 4) | (self.system[8] >> 4) << 4,
            (self.system[10] >> 4) | (self.system[10] >> 4) << 4,
            (self.system[12] >> 4) | (self.system[12] >> 4) << 4,
            0xff,
        ]
    }

    fn get_color1(&self) -> [u8; 4] {
        [
            (self.system[8] << 4) | (self.system[8] << 4) >> 4,
            (self.system[10] << 4) | (self.system[10] << 4) >> 4,
            (self.system[12] << 4) | (self.system[12] << 4) >> 4,
            0xff,
        ]
    }

    fn get_color2(&self) -> [u8; 4] {
        [
            (self.system[9] >> 4) | (self.system[9] >> 4) << 4,
            (self.system[11] >> 4) | (self.system[11] >> 4) << 4,
            (self.system[13] >> 4) | (self.system[13] >> 4) << 4,
            0xff,
        ]
    }

    fn get_color3(&self) -> [u8; 4] {
        [
            (self.system[9] << 4) | (self.system[9] << 4) >> 4,
            (self.system[11] << 4) | (self.system[11] << 4) >> 4,
            (self.system[13] << 4) | (self.system[13] << 4) >> 4,
            0xff,
        ]
    }

    fn draw_screen_bg(&mut self, x: u16, y: u16, color: [u8; 4]) {
        let base: usize = ((x as usize) + (y as usize * SCREEN_WIDTH)) * 4;
        self.screen_buffer_bg[base] = color[0];
        self.screen_buffer_bg[base + 1] = color[1];
        self.screen_buffer_bg[base + 2] = color[2];
        self.screen_buffer_bg[base + 3] = color[3];
    }

    fn draw_screen_fg(&mut self, x: u16, y: u16, color: [u8; 4]) {
        let base: usize = ((x as usize) + (y as usize * SCREEN_WIDTH)) * 4;
        self.screen_buffer_fg[base] = color[0];
        self.screen_buffer_fg[base + 1] = color[1];
        self.screen_buffer_fg[base + 2] = color[2];
        self.screen_buffer_fg[base + 3] = color[3];
    }

    pub fn write_short(&mut self, val: u16, device: u8, mem: &Vec<u8>) {
        let next_device = device + 1;
        self.write((val / 256) as u8, device, mem);
        self.write((val % 256) as u8, next_device, mem);
    }

    pub fn read(&self, device: u8) -> u8 {
        match Device::from(device) {
            Device::ScreenXHigh => self.screen[7],
            Device::ScreenXLow => self.screen[8],
            Device::ScreenYHigh => self.screen[9],
            Device::ScreenYLow => self.screen[10],
	    Device::ScreenWidthHigh => SCREEN_WIDTH_HIGH,
	    Device::ScreenWidthLow => SCREEN_WIDTH_LOW,
	    Device::ScreenHeightHigh => SCREEN_HEIGHT_HIGH,
	    Device::ScreenHeightLow => SCREEN_HEIGHT_LOW,
	    Device::ControllerButton => self.controller[2],
	    Device::ControllerKey => self.controller[3],
            _ => todo!(),
        }
    }

    pub fn read_short(&self, device: u8) -> u16 {
        let high = self.read(device) as u16;
        let low = self.read(device + 1) as u16;
        high * 256 + low
    }
}
