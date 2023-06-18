use fltk::{window::Window, app::{App, Scheme}, prelude::{WidgetBase, GroupExt, WidgetExt, WindowExt}};
use pixels::{Pixels, SurfaceTexture};

use crate::machine::{SCREEN_WIDTH, SCREEN_HEIGHT, TEXT_WIDTH, TEXT_HEIGHT, SCREEN_BUFFER_1, SCREEN_BUFFER_2, TEXT_BUFFER_1, TEXT_BUFFER_2, BITMAP, FLAG_BIT_E, FLAG_BIT_B};

/// Screen uses raw pointers and needs to be handled in the Machine's drop function
pub(crate) struct Screen {
    pub(crate) machine_alive: bool,
    pub(crate) screen_alive: bool,

    scale: usize,

    screen_buffer_1: &'static [u32;SCREEN_WIDTH*SCREEN_HEIGHT],
    screen_buffer_2: &'static [u32;SCREEN_WIDTH*SCREEN_HEIGHT],
    text_buffer_1: &'static [u8;TEXT_WIDTH*TEXT_HEIGHT],
    text_buffer_2: &'static [u8;TEXT_WIDTH*TEXT_HEIGHT],
    bitmap_font: &'static [u8;256*8*8],
    flag_reg: &'static u32
}

impl Screen {
    pub(crate) fn new(mem_ptr: usize, flag_reg_ptr: usize, scale: usize, title: &'static str) -> Self {
        let mut self_screen = Self {
            machine_alive: true,
            screen_alive: true,

            scale,

            screen_buffer_1: unsafe { &*((mem_ptr + SCREEN_BUFFER_1) as *mut _) },
            screen_buffer_2: unsafe { &*((mem_ptr + SCREEN_BUFFER_2) as *mut _) },
            text_buffer_1: unsafe { &*((mem_ptr + TEXT_BUFFER_1) as *mut _) },
            text_buffer_2: unsafe { &*((mem_ptr + TEXT_BUFFER_2) as *mut _) },
            bitmap_font: unsafe { &*((mem_ptr + BITMAP) as *mut _) },
            flag_reg: unsafe { &*(flag_reg_ptr as *mut _) }
        };
        
        let screen = unsafe { &mut *(&mut self_screen as *mut Screen) };
        
        std::thread::spawn(move || {
            let app = App::default();
            let mut window = Window::new(100 as i32, 100 as i32, (SCREEN_WIDTH * scale) as i32, (SCREEN_HEIGHT * scale) as i32, title);
            window.make_resizable(false);
            window.end();
            window.show();

            let mut pixels = {
                let pixel_width = window.pixel_w() as u32;
                let pixel_height = window.pixel_h() as u32;
                let surface_texture = SurfaceTexture::new(pixel_width, pixel_height, &window);
        
                Pixels::new((SCREEN_WIDTH * scale) as u32, (SCREEN_HEIGHT * scale) as u32, surface_texture).unwrap()
            };
            
            while app.wait() {
                if !screen.machine_alive {
                    screen.screen_alive = false;
                    return;
                }
                screen.render(pixels.frame_mut());
            }
            screen.screen_alive = false;
        });
        self_screen
    }

    fn render(&mut self, pix: &mut [u8]) {
        if self.flag_reg & FLAG_BIT_E > 0 {
            self.render_screen(pix)
        } else {
            self.render_text(pix)
        }
    }

    fn render_screen(&mut self, pix: &mut [u8]) {
        let buffer = if self.flag_reg & FLAG_BIT_B > 0{
            self.screen_buffer_1
        } else {
            self.screen_buffer_2
        };
    }

    fn render_text(&mut self, pix: &mut [u8]) {
        let buffer = if self.flag_reg & FLAG_BIT_B > 0{
            self.text_buffer_1
        } else {
            self.text_buffer_2
        };
        for y in 0..TEXT_HEIGHT {
            for x in 0..TEXT_WIDTH {
                let i = y * TEXT_WIDTH + x;
                let char = buffer[i] as usize;
                for dy in 0..8 {
                    for dx in 0..8 {
                        let a = self.bitmap_font[char * 256 + dy * 8 + dx];
                        for py in 0..self.scale {
                            for px in 0..self.scale {
                                let sx = x * 8 * self.scale + dx * self.scale + px;
                                let sy = y * 8 * self.scale + dy * self.scale + py;
                                let pi = sy * SCREEN_WIDTH * sy;
                                pix[pi] = 255;
                                //image[XY(x * 8 * self.scale + dx * self.scale + px, y * 8 * self.scale + dy * self.scale + py)] = Color {
                                //    r: pix,
                                //    g: 128,
                                //    b: 255
                                //}
                            }    
                        }
                    }
                }
            }
        }
    }
}