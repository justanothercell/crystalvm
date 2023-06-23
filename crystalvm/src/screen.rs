use std::{sync::{Mutex, Arc}, process::exit, collections::VecDeque};

use fltk::{window::Window, app::{App, self}, prelude::{GroupExt, WidgetExt}};
use pixels::{Pixels, SurfaceTexture};

use crate::{machine::{SCREEN_WIDTH, SCREEN_HEIGHT, TEXT_WIDTH, TEXT_HEIGHT, SCREEN_BUFFER_1, SCREEN_BUFFER_2, TEXT_BUFFER_1, TEXT_BUFFER_2, BITMAP, FLAG_BIT_E, FLAG_BIT_B}, device::Device};

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
    pub(crate) fn create(mem_ptr: usize, flag_reg_ptr: usize, scale: usize, title: &'static str) -> (Arc<Mutex<ScreenLifetime>>, Keyboard, Mouse) {
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

        let key_data = Arc::new(Mutex::new((0, 0, 0)));
        let keyboard = Keyboard {
            key_data: key_data.clone(),
            sending: "keyboard\0\0\0\0\0\0\0\0".bytes().collect()
        };
        let mouse_data = Arc::new(Mutex::new((0, 0, 0)));
        let mouse = Mouse {
            mouse_data: mouse_data.clone(),
            sending: "mouse\0\0\0\0\0\0\0\0\0\0\0".bytes().collect()
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
                app::awake();
                let (x, y) = app::get_mouse();
                let (x, y) = (x as u32 / scale as u32, y as u32 / scale as u32);
                let (x, y) = (x.max(0).min(SCREEN_WIDTH as u32 - 1), y.max(0).min(SCREEN_HEIGHT as u32 - 1));
                *mouse_data.lock().unwrap() = (x, y, unsafe { std::mem::transmute::<i32, u32>(app::event_state().bits()) } >> 24);

                let key = unsafe { std::mem::transmute::<i32, u32>(app::event_original_key().bits()) };
                let text = app::event_text().replace('\r', "\n");
                let text = if let Some(c) = text.chars().next() { c as u8 } else { 0 };
                let flags = (app::event_key_down(app::event_original_key()) as u8) << 7 | (app::is_event_alt() as u8) << 3 | (app::is_event_command() as u8) << 2 | (app::is_event_ctrl() as u8) << 1 | (app::is_event_shift() as u8) << 0;
                *key_data.lock().unwrap() = (key, text, flags);
            }
            life.lock().unwrap().screen_alive = false;
            exit(0);
        });
        (screen_life, keyboard, mouse)
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
                        
                        pix[pi*4] = l;
                        pix[pi*4+1] = l;
                        pix[pi*4+2] = l;
                        pix[pi*4+3] = 255;
                    }    
                }
            }
        }
    }
}

pub struct Keyboard {
    sending: VecDeque<u8>,
    key_data: Arc<Mutex<(u32, u8, u8)>>
}

impl Device for Keyboard {
    fn write_byte(&mut self) -> Option<u8> {
        if self.sending.len() == 0 {
            let (key, text, flags) = *self.key_data.lock().unwrap();
            self.sending.append(&mut key.to_be_bytes().into_iter().collect());
            self.sending.append(&mut text.to_be_bytes().into_iter().collect());
            self.sending.append(&mut flags.to_be_bytes().into_iter().collect());
            self.sending.push_back(0);
        }
        let c = self.sending.pop_front();
        if let Some(c) = c {
            println!("{}", char::from(c));
        }
        c
    }

    fn receive_byte(&mut self, _: u8) {

    }
}

pub struct Mouse {
    sending: VecDeque<u8>,
    mouse_data: Arc<Mutex<(u32, u32, u32)>>,
}

impl Device for Mouse {
    fn write_byte(&mut self) -> Option<u8> {
        if self.sending.len() == 0 {
            let (x, y, mask) = *self.mouse_data.lock().unwrap();
            self.sending.append(&mut x.to_be_bytes().into_iter().collect());
            self.sending.append(&mut y.to_be_bytes().into_iter().collect());
            self.sending.push_back(0);
        }
        self.sending.pop_front()
    }

    fn receive_byte(&mut self, _: u8) {

    }
}