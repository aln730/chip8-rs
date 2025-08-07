use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const EXTENDED_WIDTH: usize = 128;
pub const EXTENDED_HEIGHT: usize = 64;

let mut gfx = [[0u8; EXTENDED_WIDTH]; EXTENDED_HEIGHT];
let mut high_res_mode = false;

const SCALE: u32 = 10;

pub struct Display {
    canvas: Canvas<Window>,
    pub high_res_mode: bool,
}

impl Display {
    pub fn new(sdl_context: &Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("CHIP-8-RS", (EXTENDED_WIDTH as u32) * SCALE, (EXTENDED_HEIGHT as u32) * SCALE)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().present_vsync().build().unwrap();

        Display {
            canvas,
            high_res_mode: false,
        }

    pub fn draw(&mut self, gfx: &[u8]) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        self.canvas.set_draw_color(Color::WHITE);

        let (width, height) = if self.high_res_mode {
            (EXTENDED_WIDTH, EXTENDED_HEIGHT)
        } else {
            (SCREEN_WIDTH, SCREEN_HEIGHT)
        };

        for (i, &pixel) in gfx.iter().enumerate(){
            if pixel == 1 {
                let x = (i as u32) % width;
                let y = (i as u32) / width;

                let rect = Rect::new(
                    (x * SCALE) as i32,
                    (y * SCALE) as i32,
                    SCALE,
                    SCALE,
                );

                self.canvas.fill_rect(rect).unwrap();
            }
        }

        self.canvas.present();
    }
}
}