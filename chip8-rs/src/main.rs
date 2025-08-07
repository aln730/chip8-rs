mod chip8;
mod display;

use chip8::{Chip8, Quirks};
use display::Display;
use std::time::{Duration, Instant};
use std::thread::sleep;

mod input;
use input::map_keycode;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let mut chip8 = Chip8::new(Quirks {
        shift_uses_vy: false,
        bnnn_uses_vx: false,
        fx55_increases_i: false,
    });

    let _ = chip8.load_rom("roms/spaceinvaders.ch8");

    let mut display = Display::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let frame_duration = Duration::from_micros(16667);

    'running: loop {
        let frame_start = Instant::now();

        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some(mapped) = map_keycode(key) {
                        chip8.keypad[mapped] = true;
                    }
                }

                Event::KeyUp { keycode: Some(key), ..} => {
                    if let Some(mapped) = map_keycode(key) {
                        chip8.keypad[mapped] = false;
                    }
                }

                _ => {}
            }
        }

        chip8.emulate_cycle();

        if chip8.draw_flag {
            display.draw(&chip8.gfx);
            chip8.draw_flag = false;
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            sleep(frame_duration - elapsed);
        }
    }
}
