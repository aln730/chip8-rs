use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

pub const SCREEN_WIDTH: u32 = 64;
pub const SCREEN_HEIGHT: u32 = 32;
const SCALE: u32 = 10;

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl_context: &Sdl) -> Self {
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8-RS", SCREEN_WIDTH * SCALE, SCREEN_HEIGHT * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().present_vsync().build().unwrap();



        Display { canvas }
    }

    pub fn draw(&mut self, gfx: &[u8; (SCREEN_HEIGHT * SCREEN_WIDTH) as usize]) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        self.canvas.set_draw_color(Color::WHITE);

        for (i, &pixel) in gfx.iter().enumerate(){
            if pixel == 1 {
                let x = (i as u32) % SCREEN_WIDTH;
                let y = (i as u32) / SCREEN_WIDTH;

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