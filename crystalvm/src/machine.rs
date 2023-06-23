use std::{path::Path, fs::File, io::{Seek, Read}, sync::{Arc, Mutex, RwLock}, collections::HashMap, hash::BuildHasher, panic::PanicInfo, rc::Rc};
use std::ops::*;

use crate::{screen::{Screen, ScreenLifetime}, device::{Device, self}};

/// STACK
pub const REG_S: usize = 48;
/// INSTR PTR
pub const REG_I: usize = 49;
/// FRAME PTR
pub const REG_L: usize = 50;
/// CARRY
pub const REG_C: usize = 51;
/// FLAG
pub const REG_F: usize = 52;
/// INTERRUPT ID
pub const REG_Q: usize = 53;
// INTERRUPT ID
pub const REG_D: usize = 54;

pub const NUM_REGS: usize = 55;

/// zero: Z = a != b
pub const FLAG_BIT_Z: u32 = 0b00000000_00000000_00000000_00000001;
/// sign: S = a < b
pub const FLAG_BIT_S: u32 = 0b00000000_00000000_00000000_00000010;
/// carry: an operation over or underflowed/produced a carry value
pub const FLAG_BIT_C: u32 = 0b00000000_00000000_00000000_00000100;
/// carry in: whether to include the carry of the last operation in this operation
pub const FLAG_BIT_M: u32 = 0b00000000_00000000_10000000_00000000;
pub const FLAG_BIT_E: u32 = 0b00000000_01000000_00000000_00000000;
pub const FLAG_BIT_B: u32 = 0b00000000_10000000_00000000_00000000;

pub const INTERRUPT_DEVICE_ATTACH: u32 = 0b00000000;
pub const INTERRUPT_DEVICE_READ_FINISHED: u32 = 0b00000000;
pub const INTERRUPT_DEVICE_WRITE_FINISHED: u32 = 0b00000000;
pub const INTERRUPT_DEVICE_DETACH: u32 = 0b00000000;
pub const INTERRUPT_DUMMY: u32 = u32::MAX;

pub const IMAGE_BASE: usize = 0x0008DE00;
pub const INTERRUPT_HANDLER: usize = 0x0008DE00;
pub const ENTRYPOINT: usize = 0x0008E000;

pub const SCREEN_BUFFER_1: usize = 0x00000000;
pub const SCREEN_BUFFER_2: usize = 0x0003E800;
pub const TEXT_BUFFER_1: usize = 0x0007D000;
pub const TEXT_BUFFER_2: usize = 0x0007D3E8;
pub const BITMAP: usize = 0x0007D800;

pub const SCREEN_WIDTH: usize = 320;
pub const SCREEN_HEIGHT: usize = 200;
pub const TEXT_WIDTH: usize = 40;
pub const TEXT_HEIGHT: usize = 25;

pub struct Machine {
    pub(crate) memory: Box<Vec<u8>>,
    pub(crate) registers: Box<[u32; NUM_REGS]>,
    next_device_id: u32,
    interrupt_wait_counter: u32,
    screen_life: Arc<Mutex<ScreenLifetime>>,
    devices: DeviceMap,

    default_panic_hook: Arc<Box<dyn for<'a, 'b> std::ops::Fn(&'a PanicInfo<'b>) + Send + Sync>>
}

impl Machine {
    pub fn from_image<P: AsRef<Path>>(path: P, memory_size: usize, window_title: &'static str, window_scale: usize) -> Self {
        let mut image = File::open(path).unwrap();
        let img_size = image.stream_len().unwrap() as usize;
        let mut image_contents = Vec::with_capacity(img_size);
        image.read_to_end(&mut image_contents).unwrap();
        if memory_size < img_size + IMAGE_BASE {
            panic!("need at least 0x{:X} memory cells, only got 0x{:X} supplied", img_size + IMAGE_BASE, memory_size)
        }
        let mut memory = Box::new(Vec::with_capacity(memory_size));
        unsafe{ 
            memory.set_len(memory_size);
            std::ptr::copy_nonoverlapping(image_contents.as_ptr(), (memory.as_mut_ptr() as usize + IMAGE_BASE) as *mut u8, image_contents.len());
            std::ptr::copy_nonoverlapping(include_bytes!("../target/font.rbmf").as_ptr(), (memory.as_mut_ptr() as usize + BITMAP) as *mut u8, 256*64);
        }
        let registers = Box::new([0;NUM_REGS]);

        let prev_hook = Arc::new(std::panic::take_hook());
        let prev = prev_hook.clone();
        let reg_ptr = &registers as *const _ as usize;
        std::panic::set_hook(Box::new(move |info| {
            println!("Panicked! Registers: {:X?}", unsafe {&*(reg_ptr as *const Box<[u32; NUM_REGS]>)});
            prev(info);
        }));
        let mut machine = Machine {  
            screen_life: Screen::create(memory.as_ptr() as usize, &registers[REG_F] as *const _ as usize, window_scale, window_title),
            memory,
            registers,
            next_device_id: 1,
            interrupt_wait_counter: 0,
            devices: Default::default(),

            default_panic_hook: prev_hook
        };
        machine.registers[REG_I] = ENTRYPOINT as u32;
        machine
    }

    pub(crate) fn execute_next(&mut self) {
        let raw = self.read_word(self.registers[REG_I]);
        let instr = raw >> 21;
        let arg0 = (raw >> 14 & 0b01111111) as u8;
        let arg1 = (raw >> 7 & 0b01111111) as u8;
        let arg2 = (raw & 0b01111111) as u8;
        //println!("{instr:011b} {arg0:07b} {arg1:07b} {arg2:07b}");
        macro_rules! linear_instr {
            ($logic: expr) => {{
                $logic;
                self.registers[REG_I] += 4;
            }};
        }
        macro_rules! arith_instr {
            ($ty: ident::$fun: ident (a) ) => { linear_instr!{ unsafe {
                let a: $ty = std::mem::transmute(self.fetch_data(arg0));
                self.set_data(arg1, std::mem::transmute($ty::$fun(a)))
            } }};
            ($ty: ident::$fun: ident (a, b) ) => { linear_instr!{ unsafe {
                let a: $ty = std::mem::transmute(self.fetch_data(arg0));
                let b: $ty = std::mem::transmute(self.fetch_data(arg1));
                self.set_data(arg2, std::mem::transmute($ty::$fun(a, b)))
            } }};
            ($ty: ident::$fun: ident (a, b) carry bool) => { linear_instr!{ unsafe {
                let a: $ty = std::mem::transmute(self.fetch_data(arg0));
                let b: $ty = std::mem::transmute(self.fetch_data(arg1));
                let carry_in = self.registers[REG_F] & FLAG_BIT_M > 0;
                let (r, carry) = $ty::$fun(a, b, if carry_in { self.registers[REG_C] & 0b1 > 0 } else { false });
                self.set_data(arg2, std::mem::transmute(r));
                self.registers[REG_C] = carry as u32;
                self.registers[REG_F] = (self.registers[REG_F] & !FLAG_BIT_C) | ((carry as u32) * FLAG_BIT_C);
            } }};
            ($ty: ident::$fun: ident (a, b) carry self) => { linear_instr!{ unsafe {
                let a: $ty = std::mem::transmute(self.fetch_data(arg0));
                let b: $ty = std::mem::transmute(self.fetch_data(arg1));
                let carry_in = self.registers[REG_F] & FLAG_BIT_M > 0;
                let (r, carry) = $ty::$fun(a, b, if carry_in { std::mem::transmute(self.registers[REG_C]) } else { std::mem::transmute(0) });
                self.set_data(arg2, std::mem::transmute(r));
                self.registers[REG_C] = std::mem::transmute(carry);
                self.registers[REG_F] = (self.registers[REG_F] & !FLAG_BIT_C) | (((carry != 0) as u32) * FLAG_BIT_C);
            } }};
            ($ty: ident::$fun: ident (a, b) overflow) => { linear_instr!{ unsafe {
                let a: $ty = std::mem::transmute(self.fetch_data(arg0));
                let b: $ty = std::mem::transmute(self.fetch_data(arg1));
                let (r, carry) = $ty::$fun(a, b);
                self.set_data(arg2, std::mem::transmute(r));
                self.registers[REG_C] = carry as u32;
                self.registers[REG_F] = (self.registers[REG_F] & !FLAG_BIT_C) | ((carry as u32) * FLAG_BIT_C);
            } }};

            ($ty: ident |$a: ident| $expr: expr) => { linear_instr!{ unsafe {
                let $a: $ty = std::mem::transmute(self.fetch_data(arg0));
                self.set_data(arg1, std::mem::transmute($expr))
            } }};

            ($tya: ident $tyb: ident |$a: ident, $b: ident| $expr: expr) => { linear_instr!{ unsafe {
                let $a: $tya = std::mem::transmute(self.fetch_data(arg0));
                let $b: $tyb = std::mem::transmute(self.fetch_data(arg1));
                self.set_data(arg2, std::mem::transmute($expr))
            } }};
        }
        macro_rules! cmp {
            ($ty: ident) => { linear_instr!{ unsafe {
                let a: $ty = std::mem::transmute(self.fetch_data(arg0));
                let b: $ty = std::mem::transmute(self.fetch_data(arg1));
                self.registers[REG_F] = (self.registers[REG_F] & !FLAG_BIT_Z) | (((a == b) as u32) * FLAG_BIT_Z);  // zero/eq
                self.registers[REG_F] = (self.registers[REG_F] & !FLAG_BIT_S) | (((a < b) as u32) * FLAG_BIT_S);  // less
            } }};
        }
        macro_rules! jump_if {
            (|$a: ident| $cond: expr) => {{
                let $a = self.registers[REG_F];
                let dest = self.fetch_data(arg0);
                if $cond {
                    self.registers[REG_I] = dest;
                } else {
                    self.registers[REG_I] += 4;
                }
            }};
        }
        match instr {
            0b000_00000000 => linear_instr!(()), // noop
            0b000_00000001 => arith_instr!(u32::carrying_add(a, b) carry bool), // addu
            0b000_00000010 => arith_instr!(u32::borrowing_sub(a, b) carry bool), // subu
            0b000_00000011 => arith_instr!(u32::carrying_mul(a, b) carry self), // mulu
            0b000_00000100 => arith_instr!(u32::div(a, b)), // divu
            0b000_00000101 => arith_instr!(u32::rem(a, b)), // modu
            0b000_00000111 => cmp!(u32), // cmpu
            0b000_00010001 => arith_instr!(i32::carrying_add(a, b) carry bool), // addi
            0b000_00010010 => arith_instr!(i32::borrowing_sub(a, b) carry bool), // subi
            0b000_00010011 => arith_instr!(i32::overflowing_mul(a, b) overflow), // muli
            0b000_00010100 => arith_instr!(i32::div(a, b)), // divi
            0b000_00010101 => arith_instr!(i32::rem(a, b)), // modi
            0b000_00010111 => cmp!(i32), // cmpi
            0b000_00011000 => arith_instr!(i32::abs(a)), // absi
            0b000_00011001 => arith_instr!(i32 u32 |a, b| i32::pow(a, b)), // powi
            0b000_00001000 => arith_instr!(u32::bitand(a, b)), // and
            0b000_00001001 => arith_instr!(u32::bitor(a, b)), // or
            0b000_00001010 => arith_instr!(u32::bitxor(a, b)), // xor
            0b000_00001011 => arith_instr!(u32::not(a)), // not
            0b000_00001100 => arith_instr!(u32::shl(a, b)), // shl
            0b000_00001101 => arith_instr!(u32::shr(a, b)), // shr
            0b000_00001110 => arith_instr!(u32::rotate_left(a, b)), // rol
            0b000_00001111 => arith_instr!(u32::rotate_right(a, b)), // ror
            0b000_00011100 => arith_instr!(i32 |a| a as u32), // itu
            0b000_00011101 => arith_instr!(u32 |a| a as i32), // uti
            0b000_00011110 => arith_instr!(i32 |a| a as f32), // itf
            0b000_00011111 => arith_instr!(f32 |a| a as i32), // fti
            0b000_00100000 => arith_instr!(f32::add(a, b)), // addf
            0b000_00100001 => arith_instr!(f32::sub(a, b)), // subf
            0b000_00100010 => arith_instr!(f32::mul(a, b)), // mulf
            0b000_00100011 => arith_instr!(f32::div(a, b)), // divf
            0b000_00100100 => arith_instr!(f32::rem(a, b)), // modf
            0b000_00100101 => arith_instr!(f32::abs(a)), // absf
            0b000_00100110 => arith_instr!(f32 i32 |a, b| f32::powi(a, b)), // powfi
            0b000_00110110 => arith_instr!(f32::powf(a, b)), // powfi
            0b000_00100111 => cmp!(f32), // cmpf
            0b000_00101000 => arith_instr!(f32::sqrt(a)), // sqrt
            0b000_00101001 => arith_instr!(f32::exp(a)), // exp
            0b000_00101010 => arith_instr!(f32::log(a, b)), // log
            0b000_00111010 => arith_instr!(f32::ln(a)), // ln
            0b000_00101011 => arith_instr!(f32::sin(a)), // sin
            0b000_00101100 => arith_instr!(f32::asin(a)), // asin
            0b000_00101101 => arith_instr!(f32::cos(a)), // cos
            0b000_00101110 => arith_instr!(f32::tan(a)), // tan
            0b000_00101111 => arith_instr!(f32::tan(a)), // atan
            0b000_00110000 => arith_instr!(f32::sinh(a)), // sinh
            0b000_00110001 => arith_instr!(f32::asinh(a)), // asih
            0b000_00110010 => arith_instr!(f32::cosh(a)), // cosh
            0b000_00110011 => arith_instr!(f32::acosh(a)), // acoh
            0b000_01000000 => jump_if!(|_a| true), // jmp
            0b000_01000010 => jump_if!(|a| a & FLAG_BIT_Z != 0), // jz
            0b000_01000011 => jump_if!(|a| a & FLAG_BIT_Z == 0), // jnz
            0b000_01000100 => jump_if!(|a| a & FLAG_BIT_S != 0), // jl
            0b000_01000101 => jump_if!(|a| a & FLAG_BIT_S == 0), // jnl
            0b000_01000110 => jump_if!(|a| a & FLAG_BIT_C != 0), // jc
            0b000_01000111 => jump_if!(|a| a & FLAG_BIT_C == 0), // jnc
            
            0b000_01010000 => linear_instr!{{
                let fun = self.fetch_data(arg0);
                self.call(fun);
            }}, // call
            0b000_01010001 => linear_instr!{{
                self.registers[REG_S] = self.registers[REG_L];
                self.registers[REG_I] = self.read_word(self.registers[REG_L] + 4);
                self.registers[REG_L] = self.read_word(self.registers[REG_L]);
            }}, // ret
            0b000_10000000 => linear_instr!{{
                let v = self.fetch_data(arg0);
                self.set_data(arg1, v)
            }}, // move
            0b000_10000001 => linear_instr!{{
                let a = self.fetch_data(arg1);
                let d = self.read_word(a);
                self.set_data(arg0, d)
            }}, // ld
            0b000_10000011 => linear_instr!{{
                let a = self.fetch_data(arg0);
                let d = self.fetch_data(arg1);
                self.write_word(a, d);
            }}, // st
            0b000_10000101 => linear_instr!{{
                let a = self.fetch_data(arg1);
                let d = self.memory[a as usize] as u32;
                self.set_data(arg0, d)
            }}, // ldb
            0b000_10000111 => linear_instr!{{
                let a = self.fetch_data(arg0);
                let d = self.fetch_data(arg1);
                self.memory[a as usize] = d as u8;
            }}, // stb
            0b000_10001000 => linear_instr!{{
                self.registers[REG_S] += 4;
                self.memory[self.registers[REG_S] as usize] = self.memory[self.registers[REG_S] as usize - 4];
            }}, // dup
            0b000_10001001 => linear_instr!{{
                self.registers[REG_S] += 4;
                self.memory[self.registers[REG_S] as usize] = self.memory[self.registers[REG_S] as usize - 8];
            }}, // over
            0b000_10001010 => linear_instr!{{
                let a = self.read_word(self.registers[REG_S]);
                let b = self.read_word(self.registers[REG_S] - 4);
                let c = self.read_word(self.registers[REG_S] - 8);
                self.write_word(self.registers[REG_S], b);
                self.write_word(self.registers[REG_S] - 4, c);
                self.write_word(self.registers[REG_S] - 8, a);
            }}, // srl
            0b000_10001011 => linear_instr!{{
                let a = self.read_word(self.registers[REG_S]);
                let b = self.read_word(self.registers[REG_S] - 4);
                let c = self.read_word(self.registers[REG_S] - 8);
                self.write_word(self.registers[REG_S], c);
                self.write_word(self.registers[REG_S] - 4, a);
                self.write_word(self.registers[REG_S] - 8, b);
            }}, // srr
            0b000_10001100 => linear_instr!{{
                self.registers[REG_S] += 4;
                self.write_word(self.registers[REG_S], self.registers[REG_L]);
                self.registers[REG_L] = self.registers[REG_S];
            }}, // enter
            0b000_10001101 => linear_instr!{{
                self.registers[REG_L] = self.read_word(self.registers[REG_S]);
                self.registers[REG_S] -= 4;
            }}, // leave
            0b000_10001110 => linear_instr!{{
                for i in 0..self.registers.len() {
                    self.set_data(0b01111111, self.registers[i])
                }
            }}, // pshar
            0b000_10001111 => linear_instr!{{
                for i in (0..self.registers.len()).rev() {
                    self.registers[i] = self.fetch_data(0b01111111);
                }
            }}, // resar
            0b000_11100000 => linear_instr!(()), // time
            0b000_11100001 => linear_instr!(()), // wait
            0b000_11101001 => linear_instr!{{
                let did = self.fetch_data(arg0);
                let ptr = self.fetch_data(arg1);
                let len = self.fetch_data(arg2);
                if let Some(device) = self.devices.get(&did) {
                    let mut write = device.write().unwrap();
                    write.read_pointer = ptr;
                    write.read_length = len;
                }
            }}, // dread 
            0b000_11101010 => linear_instr!{{
                let did = self.fetch_data(arg0);
                let ptr = self.fetch_data(arg1);
                let len = self.fetch_data(arg2);
                if let Some(device) = self.devices.get(&did) {
                    let mut write = device.write().unwrap();
                    write.write_pointer = ptr;
                    write.write_length = len;
                }
            }}, // dwrite 
            0b000_11101011 => linear_instr!(()), // dstate 
            0b111_11111111 => linear_instr!(()), // breakpoint 
            _ => linear_instr!(()),
        }

        if self.interrupt_wait_counter > 0 {
            self.interrupt_wait_counter -= 1;
            if self.interrupt_wait_counter == 0 {
                self.trigger_interrupt(INTERRUPT_DUMMY, 0);
            }
        } 
    }

    #[inline]
    pub(crate) fn fetch_data(&mut self, reglike: u8) -> u32{
        if reglike == 0b01111111 {
            self.registers[REG_I] += 4;
            self.read_word(self.registers[REG_I])
        }
        else if reglike & 0b01000000 > 0 {
            self.registers[REG_S] -= 4;
            self.read_word(self.registers[REG_S] + 4)
        } else {
            self.registers[reglike as usize]
        }
    }

    #[inline]
    pub(crate) fn set_data(&mut self, reglike: u8, data: u32) {
        if reglike & 0b01000000 > 0 {
            self.registers[REG_S] += 4;
            self.write_word(self.registers[REG_S], data)
        } else {
            self.registers[reglike as usize] = data
        }
    }

    #[inline]
    pub(crate) fn read_word(&self, addr: u32) -> u32 {
        unsafe { 
            let mut r = 0u32;
            std::ptr::copy_nonoverlapping((self.memory.as_ptr() as usize + addr as usize) as *const u8,  &mut r as *mut u32 as *mut _, std::mem::size_of::<u32>());
            r.swap_bytes()
        }
    }

    #[inline]
    pub(crate) fn write_word(&mut self, addr: u32, data: u32) {
        unsafe { 
            std::ptr::copy_nonoverlapping(&data.swap_bytes() as *const u32 as _, (self.memory.as_mut_ptr() as usize + addr as usize) as *mut u8, std::mem::size_of::<u32>())
        }
    }

    #[inline]
    pub(crate) fn call(&mut self, fun: u32) {
        self.write_word(self.registers[REG_S] + 4, self.registers[REG_L]);
        self.write_word(self.registers[REG_S] + 8, self.registers[REG_I] + 4);
        self.registers[REG_I] = fun;
        self.registers[REG_L] = self.registers[REG_S];
        self.registers[REG_S] += 8;
    }
    
    #[inline]
    pub(crate) fn trigger_interrupt(&mut self, interrupt_t: u32, did: u32) {
        self.registers[REG_Q] = interrupt_t;
        self.registers[REG_D] = did;
        self.call(INTERRUPT_HANDLER as u32);
    }

    pub fn attach_device<D: Device + 'static>(&mut self, device: D) -> u32 {
        let device_id = self.next_device_id;
        let attached_device = Arc::new(RwLock::new(AttachedDevice {
            alive: true,
            device: Box::new(device),
            read_pointer: 0,
            read_length: 0,
            write_pointer: 0,
            write_length: 0,
            
        }));
        let thread_read_device = attached_device.clone();
        let thread_read_memory_ptr = &self.memory as *const _ as usize;
        std::thread::spawn(move || {
            let memory = unsafe { &mut*(thread_read_memory_ptr as *mut Vec<u8>)};
            loop {
                let l = {
                    let read = thread_read_device.read().unwrap();
                    if !read.alive {
                        return;
                    }
                    read.read_length
                };
                if l > 0 {
                    let mut write = thread_read_device.write().unwrap();
                    if let Some(b) = write.device.write_byte() {
                        memory[write.read_pointer as usize] = b;
                        write.read_pointer += 1;
                        write.read_length -= 1;
                    }
                }
            }
        });
        let thread_write_device = attached_device.clone();
        let thread_write_memory_ptr = &self.memory as *const _ as usize;
        std::thread::spawn(move || {
            let memory = unsafe { &mut*(thread_write_memory_ptr as *mut Vec<u8>)};
            loop {
                let l = {
                    let read = thread_write_device.read().unwrap();
                    if !read.alive {
                        return;
                    }
                    read.write_length
                };
                if l > 0 {
                    let mut write = thread_write_device.write().unwrap();
                    let b = memory[write.write_pointer as usize];
                    write.device.receive_byte(b);
                    write.write_pointer += 1;
                    write.write_length -= 1;
                }
            }
        });
        self.devices.insert(device_id, attached_device);
        self.next_device_id += 1;
        device_id
    }

    pub fn remove_device(&mut self, device_id: u32) -> bool {
        self.devices.remove(&device_id).map(|device| {
            device.write().unwrap().alive = false
        }).is_some()
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        // tell the screen render thread to kill itself and wait for completion
        for device in self.devices.values() {
            device.write().unwrap().alive = false;
        }
        self.screen_life.lock().unwrap().machine_alive = false;
        while self.screen_life.lock().unwrap().screen_alive {}
        if !std::thread::panicking() {
            let hook = self.default_panic_hook.clone();
            std::panic::set_hook(Box::new(move |info| hook(info)));
        }
    }
}

struct AttachedDevice {
    alive: bool,
    device: Box<dyn Device>,
    read_pointer: u32,
    read_length: u32,
    write_pointer: u32,
    write_length: u32
}

unsafe impl Send for AttachedDevice {}
unsafe impl Sync for AttachedDevice {}

type DeviceMap = HashMap<u32, Arc<RwLock<AttachedDevice>>, U32HashBuilder>;

#[derive(Default)]
pub struct U32HashBuilder;

impl BuildHasher for U32HashBuilder {
    type Hasher = U32Hash;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        U32Hash {
            state: 0
        }
    }
}
pub struct U32Hash {
    state: u64,
}

impl std::hash::Hasher for U32Hash {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state = (self.state << 8) | (byte as u64);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }
}