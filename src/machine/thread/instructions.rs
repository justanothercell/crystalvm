use crate::machine::thread::{FLAG_BIT_L, FLAG_BIT_Z, FLAG_BIT_C, FLAG_BIT_S, FLAG_MASK_FLOAT_DIV_ERR};

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

pub const CONDITION: u32 = 0b0000_0000;
pub const REGISTERS: u32 = 0b1000_0000;

pub const MOD_JMP: u32 = 0b0010_0000;
pub const MOD_NOT: u32 = 0b0100_0000;
pub const MOD_CLR: u32 = 0b0110_0000;

pub const INSTR_JMP: u32 =  LOGIC_PREFIX | CONDITION | 0b0000_1111;
pub const INSTR_CMP: u32 =  LOGIC_PREFIX | CONDITION | 0b0001_1111;

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
                let $x = self.read_arg($a);
                let res = $fun;
                self.write_reg($c, res);
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident | $ax: ident, $bx: ident | $fun: expr) => { {
                let $ax = self.read_arg($a);
                let $bx = self.read_arg($b);
                let res = $fun;
                self.write_reg($c, res);
            } };
            ($a: ident, $b: ident, $c: ident, $type: ident :: $fun: ident on error $err: ident) => { {
                let a = self.read_arg($a);
                let b = self.read_arg($b);
                match $type::$fun(unsafe { std::mem::transmute::<u32, $type>(a) }, unsafe { std::mem::transmute::<u32, $type>(b) }) {
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
            INSTR_DIV => impl_arith!(a, b, c, u32::checked_div on error FLAG_BIT_L),
            INSTR_REM => impl_arith!(a, b, c, u32::checked_rem on error FLAG_BIT_L),
            INSTR_CADD => impl_arith!(a, b, c, u32::carrying_add carrying bool with carry_in),
            INSTR_CSUB => impl_arith!(a, b, c, u32::borrowing_sub carrying bool with carry_in),
            INSTR_CMUL => impl_arith!(a, b, c, u32::carrying_mul carrying u32 with carry_in),
            INSTR_IADD => impl_arith!(a, b, c, i32::overflowing_add carrying bool),
            INSTR_ISUB => impl_arith!(a, b, c, i32::overflowing_sub carrying bool),
            INSTR_IMUL => impl_arith!(a, b, c, i32::overflowing_mul carrying bool),
            INSTR_IDIV => impl_arith!(a, b, c, i32::checked_div on error FLAG_BIT_L),
            INSTR_IREM => impl_arith!(a, b, c, i32::checked_rem on error FLAG_BIT_L),
            INSTR_IREME => impl_arith!(a, b, c, i32::checked_rem_euclid on error FLAG_BIT_L),
            INSTR_ICADD => impl_arith!(a, b, c, i32::carrying_add carrying bool with carry_in),
            INSTR_ICSUB => impl_arith!(a, b, c, i32::borrowing_sub carrying bool with carry_in),
            INSTR_SHL => impl_arith!(a, b, c, u32::overflowing_shl carrying bool),
            INSTR_SHR => impl_arith!(a, b, c, u32::overflowing_shl carrying bool),
            INSTR_WSHL => impl_arith!(a, b, c, u32::wrapping_shl),
            INSTR_WSHR => impl_arith!(a, b, c, u32::wrapping_shr),
            INSTR_AND => impl_arith!(a, b, c, u32 |a, b| a & b),
            INSTR_OR => impl_arith!(a, b, c, u32 |a, b| a | b),
            INSTR_XOR => impl_arith!(a, b, c, u32 |a, b| a ^ b),
            INSTR_NEG => impl_arith!(a, c, u32 |a| !a),

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
            INSTR_JD => impl_logic!(jump a if FLAG_MASK_FLOAT_DIV_ERR),
            INSTR_JND => impl_logic!(jump a unless FLAG_MASK_FLOAT_DIV_ERR),
            INSTR_CLDF => impl_logic!(clear FLAG_MASK_FLOAT_DIV_ERR),
            INSTR_JL => impl_logic!(jump a if FLAG_BIT_L),
            INSTR_JNL => impl_logic!(jump a unless FLAG_BIT_L),
            INSTR_CLLF => impl_logic!(clear FLAG_BIT_L),

            INSTR_PRINT => { print!("{}", char::from_u32(self.read_arg(a)).unwrap_or_else(|| unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; '�' })) }

            _ => unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
        }
     }
}