use std::fs::File;
use std::io::{self, Read};

use rand::random;

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    pub gfx: [u8; 128 * 64],
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    pub keypad: [bool;16],
    pub draw_flag: bool,
    quirks: Quirks
    hires: bool,
}

pub struct  Quirks{
    pub shift_uses_vy: bool,
    pub bnnn_uses_vx: bool,
    pub fx55_increases_i: bool,
}

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7 
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

impl Chip8 {
    pub fn new(quirks: Quirks) -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(16),
            keypad: [false; 16],
            draw_flag: false,
            quirks: quirks,
        };

        
        for i in 0..FONTSET.len() {
            chip8.memory[0x050 + i] = FONTSET[i];
        }

        chip8
    }


pub fn load_rom(&mut self, path: &str) -> io::Result<()>{
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

        if 0x200 + buffer.len() > 4096 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ROM TOO LARGE TO FIT IN MEMORY",
            ));
        }

        for (i, &byte) in buffer.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }

        Ok(())

    }

pub fn emulate_cycle(&mut self) {
    let opcode = ((self.memory[self.pc as usize] as u16) << 8)
               |  (self.memory[(self.pc + 1) as usize] as u16);

    self.pc += 2;

    match opcode & 0xF000 {
        0x0000 => match opcode & 0x00FF {
            0x00E0 => {
                self.gfx = [0; 64 * 32];
                self.draw_flag = true;
            }
            0x00EE => {
                self.pc = self.stack.pop().expect("Stack underflow on RET");
            }
            _ => {
                println!("Unknown 0x0NNN call: {:#06x}", opcode);
            }
        }

        0x1000 => {
            let addr = opcode & 0x0FFF;
            self.pc = addr;
        }

        0x2000 => {
            let addr = opcode & 0x0FFF;
            self.stack.push(self.pc);
            self.pc = addr;
        }

        0x3000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let kk = (opcode & 0x00FF) as u8;
            if self.v[x] == kk {
                self.pc += 2;
            }
        }

        0x4000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let kk = (opcode & 0x00FF) as u8;
            if self.v[x] != kk {
                self.pc += 2;
            }
        }

        0x5000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            if opcode & 0x000F == 0 && self.v[x] == self.v[y] {
                self.pc += 2;
            }
        }

        0x6000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let byte = (opcode & 0x00FF) as u8;
            self.v[x] = byte;
        }

        0x7000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let byte = (opcode & 0x00FF) as u8;
            self.v[x] = self.v[x].wrapping_add(byte);
        }

        //Arithmatic Ops

        0x8000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            match opcode & 0x000F {
                0x0 => self.v[x] = self.v[y],
                0x1 => self.v[x] |= self.v[y],
                0x2 => self.v[x] &= self.v[y],
                0x3 => self.v[x] ^= self.v[y],
                0x4 => {
                    let (res, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = res;
                    self.v[0xF] = if carry { 1 } else { 0 };
                }
                0x5 => {
                    self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                }
                0x6 => {
                    if self.quirks.shift_uses_vy {
                        self.v[0xF] = self.v[y] & 1;
                        self.v[x] = self.v[y] >> 1;
                    } else {
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    }
                }
                0x7 => {
                    self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                }
                0xE => {
                    if self.quirks.shift_uses_vy {
                        self.v[0xF] = (self.v[y] >> 7) & 1;
                        self.v[x] = self.v[y] << 1;
                    } else {
                        self.v[0xF] = (self.v[x] >> 7) & 1;
                        self.v[x] <<= 1;
                    }
                }
                _ => println!("Unknown 0x8 opcode: {:#06x}", opcode),
            }
        }

        0x9000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            if opcode & 0x000F == 0 && self.v[x] != self.v[y] {
                self.pc += 2;
            }
        }

        0xA000 => {
            self.i = opcode & 0x0FFF;
        }

        0xB000 => {
            let addr = opcode & 0x0FFF;
            self.pc = if self.quirks.bnnn_uses_vx {
                addr + self.v[0] as u16
            } else {
                addr + self.v[((opcode & 0x0F00) >> 8) as usize] as u16
            };
        }

        0xC000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let kk = (opcode & 0x00FF) as u8;
            self.v[x] = random::<u8>() & kk;
        }

        //Display Op (Very Important!!!)

        0xD000 => {
            let x = self.v[((opcode & 0x0F00) >> 8) as usize] as u16;
            let y = self.v[((opcode & 0x00F0) >> 4) as usize] as u16;
            let height = (opcode & 0x000F) as u16;

            self.v[0xF] = 0;

            for row in 0..height {
                let sprite_byte = self.memory[(self.i + row) as usize];
                for col in 0..8 {
                    let sprite_pixel = (sprite_byte >> (7 - col)) & 1;
                    let px = (x + col) % 64;
                    let py = (y + row) % 32;
                    let index = (py * 64 + px) as usize;

                    if sprite_pixel == 1 {
                        if self.gfx[index] == 1 {
                            self.v[0xF] = 1;
                        }
                        self.gfx[index] ^= 1;
                    }
                }
            }

            self.draw_flag = true;
        }

        0xE000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let key = self.v[x] as usize;
            match opcode & 0x00FF {
                0x9E => {
                    if self.keypad[key] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.keypad[key] {
                        self.pc += 2;
                    }
                }
                _ => println!("Unknown 0xE opcode: {:#06x}", opcode),
            }
        }

        //Memory and Index Register Ops
        0xF000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            match opcode & 0x00FF {
                0x07 => self.v[x] = self.delay_timer,
                0x0A => {
                    let mut key_pressed = false;
                    for i in 0..16 {
                        if self.keypad[i] {
                            self.v[x] = i as u8;
                            key_pressed = true;
                            break;
                        }
                    }
                    if !key_pressed {
                        self.pc -= 2;
                    }
                }
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i = self.i.wrapping_add(self.v[x] as u16),
                0x29 => self.i = (self.v[x] as u16) * 5,
                0x33 => {
                    let val = self.v[x];
                    self.memory[self.i as usize] = val / 100;
                    self.memory[(self.i + 1) as usize] = (val % 100) / 10;
                    self.memory[(self.i + 2) as usize] = val % 10;
                }
                0x55 => {
                    for offset in 0..=x {
                        self.memory[(self.i + offset as u16) as usize] = self.v[offset];
                    }
                    if self.quirks.fx55_increases_i {
                        self.i += (x + 1) as u16;
                    }
                }
                0x65 => {
                    for offset in 0..=x {
                        self.v[offset] = self.memory[(self.i + offset as u16) as usize];
                    }
                    if self.quirks.fx55_increases_i {
                        self.i += (x + 1) as u16;
                    }
                }
                _ => println!("Unknown 0xF opcode: {:#06x}", opcode),
            }
        }

        //SUPERCHIP OPCODES

        0x00C0..=0x00CF => {
            let x = opcode & 0x00F;
            for y
        }

        _ => println!("Unknown opcode: {:#06x}", opcode),
    }

    // Timers

    if self.delay_timer > 0 {
        self.delay_timer -= 1;
    }
    if self.sound_timer > 0 {
        if self.sound_timer == 1 {
            println!("BEEP!");
        }
        self.sound_timer -= 1;
    }
}


}
