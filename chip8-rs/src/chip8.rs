use std::fs::File;
use std::io::{self, Read};

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8, 16],
    i: u16,
    pc: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    keypad: [bool;16],
    draw_flag: bool,
    quirls: Quirks,
}

pub struct  Quirks{
    shift_uses_vy: bool,
    bnnn_uses_vx: bool,
    fx55_increases_i: bool,
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
            quirls: quirks,
        };

        
        for i in 0..FONTSET.len() {
            chip8.memory[0x050 + i] = FONTSET[i];
        }

        chip8
    }
}

pub fn load_rom(&mut self, path: &str) -> io::Result<()>{
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if 0x200 + buffer.len() > 4096 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ROM TOO LARGE TO FIT IN MEMORY"
        ));

        for (i, &byte) in buffer.iter().enumerate(){
            self.memory[0x200 + 1] = byte;
        }

        Ok(())
    }
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