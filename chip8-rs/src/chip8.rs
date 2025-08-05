use std::fs::File;
use std::io::{self, Read};

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    pub gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    pub keypad: [bool;16],
    pub draw_flag: bool,
    quirks: Quirks
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

pub fn emulate_cycle(&mut self){
    let opcode = ((self.memory[self.pc as usize] as u16) << 8)
               |  (self.memory[(self.pc + 1) as usize] as u16);
    
    self.pc += 2;

    match opcode& 0xF000 {
        0x0000 => match opcode & 0x00FF {
            0x00E0 => {
                self.gfx = [0; 64 * 32];
                self.draw_flag = true;

            }
            0x00EE => {
                self.pc = self.stack.pop().expect("Stack underflow on RET");

            }
            _ => {
                println!("Unknown 0x0NNN call: {:#04x}", opcode);
            }
            
        }

        0x1000 => {
            let addr = opcode& 0x0FFF;
            self.pc = addr;
        }

        0x2000 => {
            let addr = opcode & 0x0FFF;
            self.stack.push(self.pc);
            self.pc = addr;
        }

        0x3000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let kk = ((opcode & 0x00FF)) as u8;
            if self.v[x] == kk {
                self.pc += 2;
            }
        }

        0x4000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let kk = ((opcode & 0x00FF) >> 8) as u8;
            if self.v[x] != kk {
                self.pc += 2;
            }
        }

        0x5000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            if opcode & 0x000F == 0 {
                if self.v[x] == 0 {
                    if self.v[x] == self.v[y] {
                        self.pc +=2;
                    }
                }
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

        0x8000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            self.v[x] = self.v[y];
        }

        0x8001 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            self.v[x] |= self.v[y];
        }

        0x8002 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            self.v[x] &= self.v[y];
        }

        0x8003 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            self.v[x] ^= self.v[y];
        }

        0x80004 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            let (result, carry) = self.v[x].overflowing_add(self.v[y]);
            self.v[x] = result;
            self.v[0xF] = if carry { 1 } else { 0 };

            //don't wanna overflow it
        }

        0x8005 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
            self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        }

        0x8006 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            self.v[0xF] = self.v[x] & 1;
            self.v[x] >>= 1;
        }



        0x9000 => {
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            if opcode& 0x000F == 0 {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
        }

        0xA000 => {
            self.i = opcode & 0x0FFF;
        }


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
            }
        }


        // Todo: Add more opcodes

        _ => {
            println!("Unknown opcode: {:#04x}", opcode);
        }
    }

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
