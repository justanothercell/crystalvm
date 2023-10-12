use super::ThreadCore;
use super::instructions::*;
use crate::machine::thread::{FLAG_BIT_L, FLAG_BIT_Z, FLAG_BIT_C, FLAG_BIT_S, FLAG_BIT_E, FLAG_PLACE_C, REG_I, REG_C, REG_F};
use std::io::Write;

impl ThreadCore {
    #[allow(unused)]
    pub(crate) fn exec_instr(&self) {
        let (instr, a, b, c) = Self::split_instr(self.read_u32(self.read_reg_unchecked(REG_I as u8)));
        //let instr_map = instr_id_name_map();
        //println!("{:?} {:?} {:?} {:?} {:?}", instr, instr_map.get(&instr), a, b, c);
        self.advance_ip();
        impl_instructions_match!(self, instr, a, b, c);
        //println!("{:?}", self.registers);
     }
}

