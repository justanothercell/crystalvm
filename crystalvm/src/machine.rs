use std::{path::Path, fs::File, io::{Seek, Read}, time::Duration};
use std::ops::*;

/// STACK
const REG_S: usize = 48;
// INSTR PTR
const REG_I: usize = 49;
/// FRAME PTR
const REG_L: usize = 50;
// CARRY
const REG_C: usize = 51;
// FLAG
const REG_F: usize = 52;
// INTERRUPT ID
const REG_Q: usize = 53;

pub struct Machine {
    pub(crate) memory: Vec<u8>,
    pub(crate) registers: [u32; 54],
    pub(crate) next_device_id: u32,
    pub(crate) interrupt_wait_counter: u32,
}

impl Machine {
    pub fn from_image<P: AsRef<Path>>(path: P, memory_size: usize) -> Self {
        let mut image = File::open(path).unwrap();
        let img_size = image.stream_len().unwrap() as usize;
        let mut image_contents = Vec::with_capacity(img_size);
        image.read_to_end(&mut image_contents).unwrap();
        if memory_size < img_size + 0x88000 {
            panic!("need at least {:X} memory cells, only got {:X} supplied", img_size + 0x087400, memory_size)
        }
        let mut machine = Machine {  
            memory: {
                let mut m = Vec::with_capacity(memory_size);
                unsafe{ 
                    m.set_len(memory_size);
                    std::ptr::copy_nonoverlapping(image_contents.as_mut_ptr(), (m.as_mut_ptr() as usize + 0x087400) as *mut u8, image_contents.len())
                }
                m
            },
            registers: [0;54],
            next_device_id: 1,
            interrupt_wait_counter: 0
        };
        machine.registers[REG_I] = 0x22100;
        machine
    }

    pub(crate)fn execute_next(&mut self) {
        let raw = self.read_word(self.registers[REG_I]);
        let instr = raw >> 21;
        let arg0 = (raw >> 14 & 0b01111111) as u8;
        let arg1 = (raw >> 7 & 0b01111111) as u8;
        let arg2 = (raw & 0b01111111) as u8;
        println!("{instr:011b} {arg0:02X} {arg1:02X} {arg2:02X}");
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
        macro_rules! jump_if {
            (|$a: ident| $cond: expr) => {{
                let $a = self.registers[REG_F];
                if $cond {
                    self.registers[REG_I] = self.fetch_data(arg0);
                } else {
                    self.registers[REG_I] += 4;
                }
            }};
        }
        match instr {
            0b000_00000000 => linear_instr!(()), // noop
            0b000_00000001 => arith_instr!(u32::add(a, b)), // addu
            0b000_00000010 => arith_instr!(u32::sub(a, b)), // subu
            0b000_00000011 => arith_instr!(u32::mul(a, b)), // mulu
            0b000_00000100 => arith_instr!(u32::div(a, b)), // divu
            0b000_00000101 => arith_instr!(u32::rem(a, b)), // modu
            0b000_00000111 => (), // cmpu
            0b000_00010001 => arith_instr!(i32::add(a, b)), // addi
            0b000_00010010 => arith_instr!(i32::sub(a, b)), // subi
            0b000_00010011 => arith_instr!(i32::mul(a, b)), // muli
            0b000_00010100 => arith_instr!(i32::div(a, b)), // divi
            0b000_00010101 => arith_instr!(i32::rem(a, b)), // modi
            0b000_00010111 => (), // cmpi
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
            0b000_00100111 => (), // cmpf
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
            0b000_01000010 => jump_if!(|a| a & 0b0001 != 0), // jz
            0b000_01000011 => jump_if!(|a| a & 0b0001 == 0), // jnz
            0b000_01000100 => jump_if!(|a| a & 0b0010 != 0), // jl
            0b000_01000101 => jump_if!(|a| a & 0b0010 == 0), // jnl
            0b000_01000110 => jump_if!(|a| a & 0b0100 != 0), // jc
            0b000_01000111 => jump_if!(|a| a & 0b0100 == 0), // jnc
            0b000_01001000 => jump_if!(|a| a & 0b1000 != 0), // jo
            0b000_01001001 => jump_if!(|a| a & 0b1000 == 0), // jno
            0b000_01010000 => linear_instr!{{
                let fun = self.fetch_data(arg0);
                self.write_word(self.registers[REG_S] + 4, self.registers[REG_L]);
                self.write_word(self.registers[REG_S] + 8, self.registers[REG_I] + 4);
                self.registers[REG_I] = fun;
                self.registers[REG_L] = self.registers[REG_S];
                self.registers[REG_S] += 8;
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
            0b000_10000010 => linear_instr!{{
                self.registers[REG_I] += 4;
                let d = self.read_word(self.registers[REG_I]);
                self.set_data(arg0, d)
            }}, // ldl
            0b000_10000011 => linear_instr!{{
                let a = self.fetch_data(arg1);
                let d = self.fetch_data(arg0);
                self.write_word(a, d);
            }}, // st
            0b000_10001000 => linear_instr!{{
                self.registers[REG_S] += 4;
                self.memory[self.registers[REG_S] as usize] = self.memory[self.registers[REG_S] as usize - 4];
            }}, // dup
            0b000_10001001 => linear_instr!{{
                self.registers[REG_S] += 4;
                self.memory[self.registers[REG_S] as usize] = self.memory[self.registers[REG_S] as usize - 8];
            }}, // over
            0b000_10001010 => linear_instr!{{
                let a = self.read_word(self.registers[REG_S] - 0);
                let b = self.read_word(self.registers[REG_S] - 4);
                let c = self.read_word(self.registers[REG_S] - 8);
                self.write_word(self.registers[REG_S] - 0, b);
                self.write_word(self.registers[REG_S] - 4, c);
                self.write_word(self.registers[REG_S] - 8, a);
            }}, // srl
            0b000_10001011 => linear_instr!{{
                let a = self.read_word(self.registers[REG_S] - 0);
                let b = self.read_word(self.registers[REG_S] - 4);
                let c = self.read_word(self.registers[REG_S] - 8);
                self.write_word(self.registers[REG_S] - 0, c);
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
            0b000_10001110 => (), // pshar
            0b000_10001111 => (), // resar
            0b000_11100000 => linear_instr!(std::thread::sleep(Duration::from_millis(self.fetch_data(arg0) as u64))), // sleep
            0b000_11100001 => (), // dinfo
            _ => linear_instr!(()),
        }

        if self.interrupt_wait_counter > 0 {
            self.interrupt_wait_counter -= 1;
            if self.interrupt_wait_counter == 0 {
                self.trigger_interrupt(0);
            }
        } 
    }

    #[inline]
    fn fetch_data(&mut self, reglike: u8) -> u32{
        if reglike & 0b01000000 > 0 {
            self.registers[REG_S] -= 4;
            self.read_word(self.registers[REG_S]+ 1)
        } else {
            self.registers[reglike as usize]
        }
    }

    #[inline]
    fn set_data(&mut self, reglike: u8, data: u32) {
        if reglike & 0b01000000 > 0 {
            self.registers[REG_S] += 4;
            self.write_word(self.registers[REG_S], data)
        }else {
            self.registers[reglike as usize] = data
        }
    }

    #[inline]
    fn read_word(&mut self, addr: u32) -> u32 {
        unsafe { 
            let mut r = 0;
            std::ptr::copy_nonoverlapping((self.memory.as_ptr() as usize + addr as usize) as *const u8,  &mut r as *mut u32 as *mut _, std::mem::size_of::<u32>());
            r
        }
    }

    #[inline]
    fn write_word(&mut self, addr: u32, data: u32) {
        unsafe { 
            std::ptr::copy_nonoverlapping(&data as *const u32 as _, (self.memory.as_mut_ptr() as usize + addr as usize) as *mut u8, std::mem::size_of::<u32>())
        }
    }
    
    fn trigger_interrupt(&mut self, iid: u32) {
        self.registers[REG_Q] = iid
    }
}