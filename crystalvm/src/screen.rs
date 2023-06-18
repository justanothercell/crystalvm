use std::{sync::{Mutex, Arc}, process::exit};

use fltk::{window::Window, app::{App, self}, prelude::{WidgetBase, GroupExt, WidgetExt}, enums::Event};
use pixels::{Pixels, SurfaceTexture};

use crate::{machine::{SCREEN_WIDTH, SCREEN_HEIGHT, TEXT_WIDTH, TEXT_HEIGHT, SCREEN_BUFFER_1, SCREEN_BUFFER_2, TEXT_BUFFER_1, TEXT_BUFFER_2, BITMAP, FLAG_BIT_E, FLAG_BIT_B}, screen};

/// Screen uses raw pointers and needs to be handled in the Machine's drop function
pub(crate) struct Screen {
    screen_buffer_1: &'static [u32;SCREEN_WIDTH*SCREEN_HEIGHT],
    screen_buffer_2: &'static [u32;SCREEN_WIDTH*SCREEN_HEIGHT],
    text_buffer_1: &'static [u8;TEXT_WIDTH*TEXT_HEIGHT],
    text_buffer_2: &'static [u8;TEXT_WIDTH*TEXT_HEIGHT],
    bitmap_font: &'static [u8;256*8*8],
    flag_reg: &'static u32
}

pub(crate) struct ScreenLifetime {
    pub(crate) machine_alive: bool,
    pub(crate) screen_alive: bool,
}

impl Screen {
    pub(crate) fn create(mem_ptr: usize, flag_reg_ptr: usize, scale: usize, title: &'static str) -> Arc<Mutex<ScreenLifetime>> {
        println!("{}", flag_reg_ptr);
        let life = Arc::new(Mutex::new(ScreenLifetime {
            machine_alive: true,
            screen_alive: true,
        }));
        let mut screen = Self {
            screen_buffer_1: unsafe { &*((mem_ptr + SCREEN_BUFFER_1) as *mut _) },
            screen_buffer_2: unsafe { &*((mem_ptr + SCREEN_BUFFER_2) as *mut _) },
            text_buffer_1: unsafe { &*((mem_ptr + TEXT_BUFFER_1) as *mut _) },
            text_buffer_2: unsafe { &*((mem_ptr + TEXT_BUFFER_2) as *mut _) },
            bitmap_font: unsafe { &*((mem_ptr + BITMAP) as *mut _) },
            flag_reg: unsafe { &*(flag_reg_ptr as *mut _) }
        };

        let screen_life = life.clone();
        std::thread::spawn(move || {
            let app = App::default();
            let mut window = Window::default()
                .with_size((SCREEN_WIDTH * scale) as i32, (SCREEN_HEIGHT * scale) as i32)
                .with_label(title);
            window.make_resizable(false);
            window.end();
            window.show();

            let mut pixels = {
                let pixel_width = window.pixel_w() as u32;
                let pixel_height = window.pixel_h() as u32;
                let surface_texture = SurfaceTexture::new(pixel_width, pixel_height, &window);
        
                Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
            };
            
            while app.wait() {
                if !life.lock().unwrap().machine_alive {
                    life.lock().unwrap().screen_alive = false;
                    app.quit();
                    return;
                }

                screen.render(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    app.quit();
                    panic!("{err}");
                }
                app::flush();
                app::awake()
                //println!("{:?} {:?} {:?} {:?}", app::event(), app::event_clicks(), app::event_key(), app::event_original_key());
            }
            life.lock().unwrap().screen_alive = false;
            exit(0);
        });
        screen_life
    }

    fn render(&mut self, pix: &mut [u8]) {
        println!("{}", &self.flag_reg as *const _ as usize);
        println!("{}", self.flag_reg as *const _ as usize);
        println!("{}", self.flag_reg);
        if self.flag_reg & FLAG_BIT_E > 0 {
            self.render_screen(pix)
        } else {
            self.render_text(pix)
        }
    }

    fn render_screen(&mut self, pix: &mut [u8]) {
        let buffer = if self.flag_reg & FLAG_BIT_B > 0{
            self.screen_buffer_2
        } else {
            self.screen_buffer_1
        };
        unsafe {
            std::ptr::copy_nonoverlapping(buffer.as_ptr() as *const u8, pix.as_mut_ptr() as *mut u8, pix.len());
        }
    }

    fn render_text(&mut self, pix: &mut [u8]) {
        let buffer = if self.flag_reg & FLAG_BIT_B > 0{
            self.text_buffer_2
        } else {
            self.text_buffer_1
        };
        for y in 0..TEXT_HEIGHT {
            for x in 0..TEXT_WIDTH {
                let i = y * TEXT_WIDTH + x;
                let c = buffer[i] as usize;
                for dy in 0..8 {
                    for dx in 0..8 {
                        let l = self.bitmap_font[c * 64 + dy * 8 + dx];
                        let sx = x * 8 + dx;
                        let sy = y * 8 + dy;
                        let pi = sy * SCREEN_WIDTH + sx;
                        
                        pix[pi*4+0] = l;
                        pix[pi*4+1] = l;
                        pix[pi*4+2] = l;
                        pix[pi*4+3] = 255;
                    }    
                }
            }
        }
    }
}