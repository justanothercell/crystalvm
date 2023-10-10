use crate::machine::thread::{FLAG_BIT_L, FLAG_BIT_Z, FLAG_BIT_C, FLAG_BIT_S, FLAG_MASK_FLOAT_DIV_ERR, FLAG_BIT_N};

use super::{ThreadCore, REG_I, REG_F, FLAG_BIT_E, FLAG_PLACE_Z, FLAG_PLACE_S, FLAG_PLACE_C, FLAG_PLACE_E, FLAG_PLACE_M, REG_C, FLAG_PLACE_L};

pub const INSTR_NOOP: u32 = 0b000_0000_0000;

pub const ARITH_PREFIX: u32 = 0b001 << 8;
pub const LOGIC_PREFIX: u32 = 0b010 << 8;
pub const IO_PREFIX: u32 = 0b010 << 8;

pub const MOD_DEFLT: u32 = 0b0000_0000;
pub const MOD_SIGND: u32 = 0b0100_0000;
pub const MOD_FLOAT: u32 = 0b1000_0000;

pub const NO_CY_IN: u32  = 0b0000_0000;
pub const CARRY_IN: u32  = 0b0010_0000;

pub const INSTR_ADD: u32 =   ARITH_PREFIX | MOD_DEFLT | NO_CY_IN | 1;
pub const INSTR_SUB: u32 =   ARITH_PREFIX | MOD_DEFLT | NO_CY_IN | 2;
pub const INSTR_MUL: u32 =   ARITH_PREFIX | MOD_DEFLT | NO_CY_IN | 3;
pub const INSTR_DIV: u32 =   ARITH_PREFIX | MOD_DEFLT | NO_CY_IN | 4;
pub const INSTR_REM: u32 =   ARITH_PREFIX | MOD_DEFLT | NO_CY_IN | 5;
pub const INSTR_POW: u32 =   ARITH_PREFIX | MOD_DEFLT | NO_CY_IN | 6;
 
pub const INSTR_CADD: u32 =  ARITH_PREFIX | MOD_DEFLT | CARRY_IN | 1;
pub const INSTR_CSUB: u32 =  ARITH_PREFIX | MOD_DEFLT | CARRY_IN | 2;
pub const INSTR_CMUL: u32 =  ARITH_PREFIX | MOD_DEFLT | CARRY_IN | 3;
// INSTR_CDIV: no such rust function exists
 
pub const INSTR_IADD: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 1;
pub const INSTR_ISUB: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 2;
pub const INSTR_IMUL: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 3;
pub const INSTR_IDIV: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 4;
pub const INSTR_IREM: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 5;
pub const INSTR_IREME: u32 = ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 6;
pub const INSTR_IABS: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 7;
pub const INSTR_IPOW: u32 =  ARITH_PREFIX | MOD_SIGND | NO_CY_IN | 8;
 
pub const INSTR_ICADD: u32 = ARITH_PREFIX | MOD_SIGND | CARRY_IN | 1;
pub const INSTR_ICSUB: u32 = ARITH_PREFIX | MOD_SIGND | CARRY_IN | 2;
// INSTR_ICMUL: no such rust function exists
// INSTR_ICDIV: no such rust function exists

pub const INSTR_SHL: u32 =   ARITH_PREFIX | MOD_DEFLT | 7;
pub const INSTR_SHR: u32 =   ARITH_PREFIX | MOD_DEFLT | 8;
pub const INSTR_WSHL: u32 =  ARITH_PREFIX | MOD_DEFLT | 9;
pub const INSTR_WSHR: u32 =  ARITH_PREFIX | MOD_DEFLT | 10;
pub const INSTR_AND: u32 =   ARITH_PREFIX | MOD_DEFLT | 11;
pub const INSTR_OR:  u32 =   ARITH_PREFIX | MOD_DEFLT | 12;
pub const INSTR_XOR: u32 =   ARITH_PREFIX | MOD_DEFLT | 13;
pub const INSTR_NEG: u32 =   ARITH_PREFIX | MOD_DEFLT | 14;
pub const INSTR_MIN: u32 =   ARITH_PREFIX | MOD_DEFLT | 15;
pub const INSTR_MAX: u32 =   ARITH_PREFIX | MOD_DEFLT | 16;

// MOD_FLOAT = 0b1000_0000 => 7 bits free!
pub const F_TRIG: u32 =   0b0100_0000;
pub const F_TRIGH: u32 =  0b0010_0000;
pub const F_ATRIG: u32 =  0b0110_0000;
pub const F_ATRIGH: u32 = 0b0101_0000;
pub const F_TFUN: u32 =   0b0111_0000;
// => 5 bits free for normal arith (0 - 31)
pub const T_SIN: u32 = 0b0000_0001;
pub const T_COS: u32 = 0b0000_0010;
pub const T_TAN: u32 = 0b0000_0100;


pub const INSTR_FADD: u32 =  ARITH_PREFIX | MOD_FLOAT | 1;
pub const INSTR_FSUB: u32 =  ARITH_PREFIX | MOD_FLOAT | 2;
pub const INSTR_FMUL: u32 =  ARITH_PREFIX | MOD_FLOAT | 3;
pub const INSTR_FDIV: u32 =  ARITH_PREFIX | MOD_FLOAT | 4;
pub const INSTR_FREM: u32 =  ARITH_PREFIX | MOD_FLOAT | 5;
pub const INSTR_FREME: u32 = ARITH_PREFIX | MOD_FLOAT | 5;
pub const INSTR_FABS: u32 =  ARITH_PREFIX | MOD_FLOAT | 7;
pub const INSTR_FPOWF: u32 = ARITH_PREFIX | MOD_FLOAT | 8;
pub const INSTR_FPOWI: u32 = ARITH_PREFIX | MOD_FLOAT | 9;

pub const INSTR_FLOOR: u32 = ARITH_PREFIX | MOD_FLOAT | 9;
pub const INSTR_CEIL: u32 =  ARITH_PREFIX | MOD_FLOAT | 10;
pub const INSTR_ROUND: u32 = ARITH_PREFIX | MOD_FLOAT | 11;
pub const INSTR_SIGN: u32 =  ARITH_PREFIX | MOD_FLOAT | 12;
pub const INSTR_FPART: u32 = ARITH_PREFIX | MOD_FLOAT | 13;
pub const INSTR_IPART: u32 = ARITH_PREFIX | MOD_FLOAT | 14;
pub const INSTR_RECIP: u32 = ARITH_PREFIX | MOD_FLOAT | 15;
pub const INSTR_SQRT: u32 =  ARITH_PREFIX | MOD_FLOAT | 16;
pub const INSTR_CBRT: u32 =  ARITH_PREFIX | MOD_FLOAT | 17;
pub const INSTR_EXP: u32 =   ARITH_PREFIX | MOD_FLOAT | 18;
pub const INSTR_EXP2: u32 =  ARITH_PREFIX | MOD_FLOAT | 19;
pub const INSTR_EXPM1: u32 = ARITH_PREFIX | MOD_FLOAT | 20;
pub const INSTR_LN: u32 =    ARITH_PREFIX | MOD_FLOAT | 21;
pub const INSTR_LOG2: u32 =  ARITH_PREFIX | MOD_FLOAT | 22;
pub const INSTR_LOG10: u32 = ARITH_PREFIX | MOD_FLOAT | 23;
pub const INSTR_LN1P: u32 =  ARITH_PREFIX | MOD_FLOAT | 24;
pub const INSTR_FMIN: u32 =  ARITH_PREFIX | MOD_FLOAT | 25;
pub const INSTR_FMAX: u32 =  ARITH_PREFIX | MOD_FLOAT | 26;

pub const INSTR_SIN: u32 =   ARITH_PREFIX | MOD_FLOAT | F_TRIG   | T_SIN;
pub const INSTR_ASIN: u32 =  ARITH_PREFIX | MOD_FLOAT | F_ATRIG  | T_SIN;
pub const INSTR_SINH: u32 =  ARITH_PREFIX | MOD_FLOAT | F_TRIGH  | T_SIN;
pub const INSTR_ASINH: u32 = ARITH_PREFIX | MOD_FLOAT | F_ATRIGH | T_SIN;
pub const INSTR_COS: u32 =   ARITH_PREFIX | MOD_FLOAT | F_TRIG   | T_COS;
pub const INSTR_ACOS: u32 =  ARITH_PREFIX | MOD_FLOAT | F_ATRIG  | T_COS;
pub const INSTR_COSH: u32 =  ARITH_PREFIX | MOD_FLOAT | F_TRIGH  | T_COS;
pub const INSTR_ACOSH: u32 = ARITH_PREFIX | MOD_FLOAT | F_ATRIGH | T_COS;
pub const INSTR_TAN: u32 =   ARITH_PREFIX | MOD_FLOAT | F_TRIG   | T_TAN;
pub const INSTR_ATAN: u32 =  ARITH_PREFIX | MOD_FLOAT | F_ATRIG  | T_TAN;
pub const INSTR_TANH: u32 =  ARITH_PREFIX | MOD_FLOAT | F_TRIGH  | T_TAN;
pub const INSTR_ATANH: u32 = ARITH_PREFIX | MOD_FLOAT | F_ATRIGH | T_TAN;
pub const INSTR_ATAN2: u32 = ARITH_PREFIX | MOD_FLOAT | F_TFUN | 0;
pub const INSTR_SINCOS: u32 =ARITH_PREFIX | MOD_FLOAT | F_TFUN | 1;
pub const INSTR_MAG2D: u32 = ARITH_PREFIX | MOD_FLOAT | F_TFUN | 2;

pub const CONDITION: u32 = 0b0000_0000;
pub const REGISTERS: u32 = 0b1000_0000;

pub const MOD_JMP: u32 = 0b0010_0000;
pub const MOD_NOT: u32 = 0b0100_0000;
pub const MOD_CLR: u32 = 0b0110_0000;

pub const INSTR_JMP: u32 =  LOGIC_PREFIX | CONDITION | 0b0000_1111;
pub const INSTR_CMP: u32 =  LOGIC_PREFIX | CONDITION | 0b0001_1110;
pub const INSTR_FCMP: u32 = LOGIC_PREFIX | CONDITION | 0b0001_1101;
pub const INSTR_FCHK: u32 = LOGIC_PREFIX | CONDITION | 0b0001_1111;

pub const INSTR_JZ: u32 =   LOGIC_PREFIX | CONDITION | MOD_JMP | FLAG_PLACE_Z;
pub const INSTR_JNZ: u32 =  LOGIC_PREFIX | CONDITION | MOD_NOT | FLAG_PLACE_Z;
pub const INSTR_CLZF: u32 = LOGIC_PREFIX | CONDITION | MOD_CLR | FLAG_PLACE_Z;

pub const INSTR_JS: u32 =   LOGIC_PREFIX | CONDITION | MOD_JMP | FLAG_PLACE_S;
pub const INSTR_JNS: u32 =  LOGIC_PREFIX | CONDITION | MOD_NOT | FLAG_PLACE_S;
pub const INSTR_CLSF: u32 = LOGIC_PREFIX | CONDITION | MOD_CLR | FLAG_PLACE_S;

pub const INSTR_JC: u32 =   LOGIC_PREFIX | CONDITION | MOD_JMP | FLAG_PLACE_C;
pub const INSTR_JNC: u32 =  LOGIC_PREFIX | CONDITION | MOD_NOT | FLAG_PLACE_C;
pub const INSTR_CLCF: u32 = LOGIC_PREFIX | CONDITION | MOD_CLR | FLAG_PLACE_C;

pub const INSTR_JE: u32 =   LOGIC_PREFIX | CONDITION | MOD_JMP | FLAG_PLACE_E;
pub const INSTR_JNE: u32 =  LOGIC_PREFIX | CONDITION | MOD_NOT | FLAG_PLACE_E;
pub const INSTR_CLEF: u32 = LOGIC_PREFIX | CONDITION | MOD_CLR | FLAG_PLACE_E;

pub const INSTR_JD: u32 =   LOGIC_PREFIX | CONDITION | MOD_JMP | FLAG_PLACE_M; // M flag is arbitrary pick out of M, I and N, use FLAG_MASK_FLOAT_DIV_ERR for checking
pub const INSTR_JND: u32 =  LOGIC_PREFIX | CONDITION | MOD_NOT | FLAG_PLACE_M; // M flag is arbitrary pick out of M, I and N, use FLAG_MASK_FLOAT_DIV_ERR for checking
pub const INSTR_CLDF: u32 = LOGIC_PREFIX | CONDITION | MOD_CLR | FLAG_PLACE_M; // M flag is arbitrary pick out of M, I and N, use FLAG_MASK_FLOAT_DIV_ERR for checking

pub const INSTR_JL: u32 =   LOGIC_PREFIX | CONDITION | MOD_JMP | FLAG_PLACE_L;
pub const INSTR_JNL: u32 =  LOGIC_PREFIX | CONDITION | MOD_NOT | FLAG_PLACE_L;
pub const INSTR_CLLF: u32 = LOGIC_PREFIX | CONDITION | MOD_CLR | FLAG_PLACE_L;

pub const INSTR_PRINT: u32 = IO_PREFIX | 1;

impl ThreadCore {
    #[allow(unused)]
     pub(crate) fn exec_instr(&self) {
        let (instr, a, b, c) = Self::split_instr(self.read_u32(self.read_reg_unchecked(REG_I as u8)));
        self.advance_ip();

        macro_rules! carry_handler {
            (bool $x: expr => in) => { $x & 1 != 0 };
            (bool $x: expr => out) => { $x as u32 };
            ($ty: ident $x: expr => out) => { unsafe { std::mem::transmute::<$ty, u32>($x) } };
            ($ty: ident $x: expr => in) => { unsafe { std::mem::transmute::<u32, $ty>($x) } };
        }
        macro_rules! impl_arith {
            ($a: ident, $c: ident, $type: ident | $x: ident | $fun: expr) => { {
                let $x = unsafe { std::mem::transmute::<u32, $type>(self.read_arg($a)) };
                let res = $fun;
                self.write_reg($c, unsafe { std::mem::transmute::<$type, u32>(res) });
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident | $ax: ident, $bx: ident | $fun: expr) => { {
                let $ax = unsafe { std::mem::transmute::<u32, $type>(self.read_arg($a)) };
                let $bx = unsafe { std::mem::transmute::<u32, $type>(self.read_arg($b)) };
                let res = $fun;
                self.write_reg($c, unsafe { std::mem::transmute::<$type, u32>(res) });
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident | $ax: ident, $bx: ident | $fun: expr; on error $err: ident) => { {
                let $ax = unsafe { std::mem::transmute::<u32, $type>(self.read_arg($a)) };
                let $bx = unsafe { std::mem::transmute::<u32, $type>(self.read_arg($b)) };
                match $fun {
                    Some(res) => self.write_reg($c, unsafe { std::mem::transmute::<$type, u32>(res) }),
                    None => unsafe { self.mutator().registers[REG_F as usize] |= $err }
                }
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident :: $fun: ident) => { {
                let a = self.read_arg($a);
                let b = self.read_arg($b);
                let res = $type::$fun(unsafe { std::mem::transmute::<u32, $type>(a) }, unsafe { std::mem::transmute::<u32, $type>(b) });
                self.write_reg($c, unsafe { std::mem::transmute::<$type, u32>(res) });
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident :: $fun: ident carrying $carry: ident) => { {
                let a = self.read_arg($a);
                let b = self.read_arg($b);
                let (res, overflow) = $type::$fun(unsafe { std::mem::transmute::<u32, $type>(a) }, unsafe { std::mem::transmute::<u32, $type>(b) });
                self.write_reg($c, unsafe { std::mem::transmute::<$type, u32>(res) });
                unsafe {
                    let mutor = self.mutator();
                    mutor.registers[REG_C as usize] = carry_handler!($carry overflow => out);
                    mutor.registers[REG_F as usize] |= ((self.registers[REG_C as usize] != 0) as u32) << FLAG_PLACE_C;
                }
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident :: $fun: ident carrying $carry: ident with carry_in) => { {
                let a = self.read_arg($a);
                let b = self.read_arg($b);
                let (res, overflow) = $type::$fun(
                    unsafe { std::mem::transmute::<u32, $type>(a) }, 
                    unsafe { std::mem::transmute::<u32, $type>(b) }, 
                    carry_handler!($carry self.registers[REG_C as usize] => in));
                self.write_reg($c, unsafe { std::mem::transmute::<$type, u32>(res) });
                unsafe {
                    let mutor = self.mutator();
                    mutor.registers[REG_C as usize] = carry_handler!($carry overflow => out);
                    mutor.registers[REG_F as usize] |= ((self.registers[REG_C as usize] != 0) as u32) << FLAG_PLACE_C;
                }
            } };
        }
        macro_rules! impl_logic {
            (jump $a: ident) => { {
                let a = self.read_arg($a);
                self.write_reg_unchecked(REG_I as u8, a);
            } };
            (jump $a: ident if $mask: ident) => { {
                let a = self.read_arg($a);
                if self.read_reg_unchecked(REG_F as u8) & $mask != 0 {
                    self.write_reg_unchecked(REG_I as u8, a);
                }
            } };
            (jump $a: ident unless $mask: ident) => { {
                let a = self.read_arg($a);
                if self.read_reg_unchecked(REG_F as u8) & $mask == 0 {
                    self.write_reg_unchecked(REG_I as u8, a);
                }
            } };
            (clear $mask: ident) => { {
                self.write_reg_unchecked(REG_F as u8, self.read_reg_unchecked(REG_F as u8) & !$mask)
            } };
        }
        match instr {
            INSTR_NOOP => (),
            INSTR_ADD => impl_arith!(a, b, c, u32::overflowing_add carrying bool),
            INSTR_SUB => impl_arith!(a, b, c, u32::overflowing_sub carrying bool),
            INSTR_MUL => impl_arith!(a, b, c, u32::widening_mul carrying u32),
            INSTR_DIV => impl_arith!(a, b, c, u32 |a, b| u32::checked_div(a, b); on error FLAG_BIT_L),
            INSTR_REM => impl_arith!(a, b, c, u32 |a, b| u32::checked_rem(a, b); on error FLAG_BIT_L),
            INSTR_POW => impl_arith!(a, b, c, u32 |a, b| u32::checked_pow(a, b); on error FLAG_BIT_C),
            INSTR_CADD => impl_arith!(a, b, c, u32::carrying_add carrying bool with carry_in),
            INSTR_CSUB => impl_arith!(a, b, c, u32::borrowing_sub carrying bool with carry_in),
            INSTR_CMUL => impl_arith!(a, b, c, u32::carrying_mul carrying u32 with carry_in),
            INSTR_IADD => impl_arith!(a, b, c, i32::overflowing_add carrying bool),
            INSTR_ISUB => impl_arith!(a, b, c, i32::overflowing_sub carrying bool),
            INSTR_IMUL => impl_arith!(a, b, c, i32::overflowing_mul carrying bool),
            INSTR_IDIV => impl_arith!(a, b, c, i32 |a, b| i32::checked_div(a, b); on error FLAG_BIT_L),
            INSTR_IREM => impl_arith!(a, b, c, i32 |a, b| i32::checked_rem(a, b); on error FLAG_BIT_L),
            INSTR_IREME => impl_arith!(a, b, c, i32 |a, b| i32::checked_rem_euclid(a, b); on error FLAG_BIT_L),
            INSTR_IABS => impl_arith!(a, b, i32 |a| a.abs()),
            INSTR_IPOW => {
                let a = self.read_arg(a);
                let b = self.read_arg(b);
                match i32::checked_pow(unsafe { std::mem::transmute::<u32, i32>(a) }, b) {
                    Some(res) => self.write_reg(c, unsafe { std::mem::transmute::<i32, u32>(res) }),
                    None => unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_C }
                }
            }
            INSTR_ICADD => impl_arith!(a, b, c, i32::carrying_add carrying bool with carry_in),
            INSTR_ICSUB => impl_arith!(a, b, c, i32::borrowing_sub carrying bool with carry_in),
            INSTR_SHL => impl_arith!(a, b, c, u32::overflowing_shl carrying bool),
            INSTR_SHR => impl_arith!(a, b, c, u32::overflowing_shl carrying bool),
            INSTR_WSHL => impl_arith!(a, b, c, u32::wrapping_shl),
            INSTR_WSHR => impl_arith!(a, b, c, u32::wrapping_shr),
            INSTR_AND => impl_arith!(a, b, c, u32 |a, b| a & b),
            INSTR_OR => impl_arith!(a, b, c, u32 |a, b| a | b),
            INSTR_XOR => impl_arith!(a, b, c, u32 |a, b| a ^ b),
            INSTR_NEG => impl_arith!(a, b, u32 |a| !a),
            INSTR_MIN => impl_arith!(a, b, c, u32 |a, b| u32::min(a, b)),
            INSTR_MAX => impl_arith!(a, b, c, u32 |a, b| u32::max(a, b)),

            INSTR_FADD => impl_arith!(a, b, c, f32 |a, b| a + b),
            INSTR_FSUB => impl_arith!(a, b, c, f32 |a, b| a - b),
            INSTR_FMUL => impl_arith!(a, b, c, f32 |a, b| a * b),
            INSTR_FDIV => impl_arith!(a, b, c, f32 |a, b| a / b),
            INSTR_FREM => impl_arith!(a, b, c, f32 |a, b| a % b),
            INSTR_FREME => impl_arith!(a, b, c, f32 |a, b| a.rem_euclid(b)),
            INSTR_FABS => impl_arith!(a, b, f32 |a| a.abs()),
            INSTR_FPOWF => impl_arith!(a, b, c, f32 |a, b| a.powf(b)),
            INSTR_FPOWI => {
                let a = self.read_arg(a);
                let b = self.read_arg(b);
                let res = f32::powi(unsafe { std::mem::transmute::<u32, f32>(a) }, unsafe { std::mem::transmute::<u32, i32>(b) });
                self.write_reg(c, unsafe { std::mem::transmute::<f32, u32>(res) });
            },
            INSTR_FLOOR => impl_arith!(a, b, f32 |a| f32::floor(a)),
            INSTR_CEIL => impl_arith!(a, b, f32 |a| f32::ceil(a)),
            INSTR_ROUND => impl_arith!(a, b, f32 |a| f32::round(a)),
            INSTR_SIGN => impl_arith!(a, b, f32 |a| f32::signum(a)),
            INSTR_FPART => impl_arith!(a, b, f32 |a| f32::fract(a)),
            INSTR_IPART => impl_arith!(a, b, f32 |a| f32::trunc(a)),
            INSTR_RECIP => impl_arith!(a, b, f32 |a| f32::recip(a)),
            INSTR_SQRT => impl_arith!(a, b, f32 |a| f32::sqrt(a)),
            INSTR_CBRT => impl_arith!(a, b, f32 |a| f32::cbrt(a)),
            INSTR_EXP => impl_arith!(a, b, f32 |a| f32::exp(a)),
            INSTR_EXP2 => impl_arith!(a, b, f32 |a| f32::exp2(a)),
            INSTR_EXPM1 => impl_arith!(a, b, f32 |a| f32::exp_m1(a)),
            INSTR_LN => impl_arith!(a, b, f32 |a| f32::ln(a)),
            INSTR_LOG2 => impl_arith!(a, b, f32 |a| f32::log2(a)),
            INSTR_LOG10 => impl_arith!(a, b, f32 |a| f32::log10(a)),
            INSTR_LN1P => impl_arith!(a, b, f32 |a| f32::ln_1p(a)),
            INSTR_FMIN =>  impl_arith!(a, b, c, f32 |a, b| f32::min(a, b)),
            INSTR_FMAX =>  impl_arith!(a, b, c, f32 |a, b| f32::max(a, b)),

            INSTR_SIN => impl_arith!(a, b, f32 |a| f32::sin(a)),
            INSTR_ASIN => impl_arith!(a, b, f32 |a| f32::asin(a)),
            INSTR_SINH => impl_arith!(a, b, f32 |a| f32::sinh(a)),
            INSTR_ASINH => impl_arith!(a, b, f32 |a| f32::asinh(a)),
            INSTR_COS => impl_arith!(a, b, f32 |a| f32::cos(a)),
            INSTR_ACOS => impl_arith!(a, b, f32 |a| f32::acos(a)),
            INSTR_COSH => impl_arith!(a, b, f32 |a| f32::cosh(a)),
            INSTR_ACOSH => impl_arith!(a, b, f32 |a| f32::acosh(a)),
            INSTR_TAN => impl_arith!(a, b, f32 |a| f32::tan(a)),
            INSTR_ATAN => impl_arith!(a, b, f32 |a| f32::atan(a)),
            INSTR_TANH => impl_arith!(a, b, f32 |a| f32::tanh(a)),
            INSTR_ATANH => impl_arith!(a, b, f32 |a| f32::atanh(a)),
            INSTR_ATAN2 => impl_arith!(a, b, c, f32 |a, b| f32::atan2(a, b)),
            INSTR_SINCOS => {
                let a = self.read_arg(a);
                let res = f32::sin_cos(unsafe { std::mem::transmute::<u32, f32>(a) });
                self.write_reg(b, unsafe { std::mem::transmute::<f32, u32>(res.0) });
                self.write_reg(c, unsafe { std::mem::transmute::<f32, u32>(res.1) });
            },
            INSTR_MAG2D => impl_arith!(a, b, c, f32 |a, b| f32::hypot(a, b)), 

            INSTR_JMP => impl_logic!(jump a),
            INSTR_CMP => {
                let a = self.read_arg(a);
                let b = self.read_arg(b);
                let mutor = unsafe { self.mutator() };
                if a == b { mutor.registers[REG_F as usize] |= FLAG_BIT_Z; }
                else { mutor.registers[REG_F as usize] &= !FLAG_BIT_Z; }
                if a < b { mutor.registers[REG_F as usize] |= FLAG_BIT_S; }
                else { mutor.registers[REG_F as usize] &= !FLAG_BIT_S; }
            }
            INSTR_FCMP => {
                let a = unsafe { std::mem::transmute::<u32, f32>(self.read_arg(a)) };
                let b = unsafe { std::mem::transmute::<u32, f32>(self.read_arg(b)) };
                let mutor = unsafe { self.mutator() };
                if a == b { mutor.registers[REG_F as usize] |= FLAG_BIT_Z; }
                else { mutor.registers[REG_F as usize] &= !FLAG_BIT_Z; }
                if a < b { mutor.registers[REG_F as usize] |= FLAG_BIT_S; }
                else { mutor.registers[REG_F as usize] &= !FLAG_BIT_S; }
            }
            INSTR_JZ => impl_logic!(jump a if FLAG_BIT_Z),
            INSTR_JNZ => impl_logic!(jump a unless FLAG_BIT_Z),
            INSTR_CLZF => impl_logic!(clear FLAG_BIT_Z),
            INSTR_JS => impl_logic!(jump a if FLAG_BIT_S),
            INSTR_JNS => impl_logic!(jump a unless FLAG_BIT_S),
            INSTR_CLSF => impl_logic!(clear FLAG_BIT_S),
            INSTR_JC => impl_logic!(jump a if FLAG_BIT_C),
            INSTR_JNC => impl_logic!(jump a unless FLAG_BIT_C),
            INSTR_CLCF => impl_logic!(clear FLAG_BIT_C),
            INSTR_JE => impl_logic!(jump a if FLAG_BIT_E),
            INSTR_JNE => impl_logic!(jump a unless FLAG_BIT_E),
            INSTR_CLEF => impl_logic!(clear FLAG_BIT_E),
            INSTR_JL => impl_logic!(jump a if FLAG_BIT_L),
            INSTR_JNL => impl_logic!(jump a unless FLAG_BIT_L),
            INSTR_CLLF => impl_logic!(clear FLAG_BIT_L),

            INSTR_PRINT => { print!("{}", char::from_u32(self.read_arg(a)).unwrap_or_else(|| unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; '�' })) }

            _ => unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
        }
     }
}
