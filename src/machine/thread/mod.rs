pub(crate) mod instructions;
pub(crate) mod instructions_impl;

use std::{sync::{Arc, atomic::{Ordering, AtomicU8}}, collections::HashMap};

use crate::{Machine};

use super::MachineCtx;

/// Instruction Pointer
pub const REG_I: u32 = 0x30;
/// Frame Pointer
pub const REG_W: u32 = 0x31;
/// Stack Pointer
pub const REG_S: u32 = 0x32;
/// Flags Register
pub const REG_F: u32 = 0x33;
/// Interrupting Device ID
pub const REG_D: u32 = 0x34;
/// carry/overflow/underflow/shift in/out
pub const REG_C: u32 = 0x35;

// last reg + 1
pub const NUM_REGS: u32 = 0x36;

// Flags
/// zero: Z = a == b
pub const FLAG_PLACE_Z: u32 = 0;
/// zero: Z = a == b
pub const FLAG_BIT_Z: u32 = 1 << FLAG_PLACE_Z;
/// sign: S = a < b
pub const FLAG_PLACE_S: u32 = 1;
/// sign: S = a < b
pub const FLAG_BIT_S: u32 = 1 << FLAG_PLACE_S;
/// carry: an operation over or underflowed
pub const FLAG_PLACE_C: u32 = 2;
/// carry: an operation over or underflowed
pub const FLAG_BIT_C: u32 = 1 << FLAG_PLACE_C;
/// Error flag: permission/access/out of bounds/invalid arg
pub const FLAG_PLACE_E: u32 = 3;
/// Error flag: permission/access/out of bounds/invalid arg
pub const FLAG_BIT_E: u32 = 1 << FLAG_PLACE_E;
/// floating point: -inf
pub const FLAG_PLACE_M: u32 = 4;
/// integer division by zero
pub const FLAG_PLACE_L: u32 = 5;
/// integer division by zero
pub const FLAG_BIT_L: u32 = 1 << FLAG_PLACE_L;


pub struct ThreadCore {
    machine: Arc<MachineCtx>,
    children: HashMap<u32, Arc<ThreadCore>>,

    // ready: 0, running: 1, terminating: 2, terminated: 3
    state: AtomicU8,
    access_min_addr: u32,
    access_max_addr: u32,
    permissions: u32,

    thread_id: u32,
    parent_thread_id: u32,

    registers: [u32;64]
}

impl ThreadCore {
    pub(crate) fn launch_main(machine: &mut Machine) -> u32 {
        let id = machine.ctx.next_thead_id.fetch_add(1, Ordering::SeqCst);
        if id != 0 { panic!("tried to create main thread with id {id}."); }
        let main = Arc::new(ThreadCore {
            machine: machine.ctx.clone(),
            children: Default::default(),
            state: AtomicU8::new(0),
            access_min_addr: 0,
            access_max_addr: machine.ctx.memory.len() as u32,
            permissions: !0,
            thread_id: id,
            parent_thread_id: 0,
            registers: [0u32;64],
        });
        unsafe { machine.ctx.mutator().threads.insert(id, main.clone()); }
        main.start();
        id
    }

    /// from a permission standpoint a thread is it's own child and parent
    fn is_child_of(&self, tid: u32) -> bool {
        if tid == self.thread_id { return true; }
        let mut id = self.thread_id;
        while id != 0 {
            match self.machine.threads.get(&id) {
                Some(t) => id = t.parent_thread_id,
                None => return false
            }
            if id == tid {
                return true;
            }
        }
        false
    }

    /// from a permission standpoint a thread is it's own parent and child
    fn is_parent_of(&self, tid: u32) -> bool {
        match self.machine.threads.get(&tid) {
            Some(t) => t.is_child_of(self.thread_id),
            None => return false
        }
    }

    fn start(self: Arc<Self>) {
        std::thread::spawn(move || {
            self.state.store(1, Ordering::Release);
            self.run();
            self.state.store(3, Ordering::Release);
        });
    }
    fn run(&self) {
        loop {
            // request quit?
            if self.state.load(Ordering::Relaxed) == 2 { return; }
            self.exec_instr();
        }
    }

    #[inline]
    pub(crate)fn atomic_op<F: FnOnce(u32) -> u32>(&self, addr: u32, f: F) {
        while self.machine.atomic_lock.swap(true, Ordering::SeqCst) { std::thread::yield_now() }
        f(0);
        self.machine.atomic_lock.store(false, Ordering::SeqCst);
    }
    #[inline]
    pub(crate)fn read_u8(&self, addr: u32) -> u8 {
        if addr >= self.access_min_addr && addr < self.access_max_addr {
            self.machine.memory[addr as usize]
        } else {
            unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
            0
        }
    }
    #[inline]
    pub(crate)fn write_u8(&self, addr: u32, value: u8) {
        if addr >= self.access_min_addr && addr < self.access_max_addr {
            self.machine.mem_mut()[addr as usize] = value;
        } else {
            unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
        }
    }
    #[inline]
    pub(crate)fn read_u32(&self, addr: u32) -> u32 {
        if addr >= self.access_min_addr && addr + 3 < self.access_max_addr {
            u32::from_le_bytes(self.machine.memory[addr as usize .. addr as usize + 4].try_into().unwrap())
        } else {
            unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
            0
        }
    }
    #[inline]
    pub(crate)fn write_u32(&self, addr: u32, value: u32) {
        if addr >= self.access_min_addr && addr + 3 < self.access_max_addr {
            let value = value.to_le_bytes();
            unsafe { std::ptr::copy_nonoverlapping(&value as *const [u8;4] as *mut _, (self.machine.mem_mut() as *mut _ as usize + addr as usize) as *mut u8, std::mem::size_of::<u32>()); }
        } else {
            unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
        }
    }
    #[inline]
    pub(crate)fn read_f32(&self, addr: u32) -> f32 {
        if addr >= self.access_min_addr && addr + 3 < self.access_max_addr {
            unsafe { f32::from_le_bytes(self.machine.memory[addr as usize .. addr as usize + 4].try_into().unwrap_unchecked()) }
        } else {
            unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
            0.0
        }
    }
    #[inline]
    pub(crate)fn write_f32(&self, addr: u32, value: f32) {
        if addr >= self.access_min_addr && addr + 3 < self.access_max_addr {
            let value = value.to_le_bytes();
            unsafe { std::ptr::copy_nonoverlapping(&value as *const [u8;4] as *mut _, (self.machine.mem_mut() as *mut _ as usize + addr as usize) as *mut u8, std::mem::size_of::<u32>()); }
        } else {
            unsafe { self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
        }
    }
    #[inline]
    pub(crate) fn read_arg(&self, reg: u8) -> u32 {
        unsafe { 
            let mutor = self.mutator();
            if reg == 0b0111_1111 {
                let v = self.read_u32(self.read_reg_unchecked(REG_I as u8));
                self.advance_ip();
                return v;
            }
            if reg < NUM_REGS as u8 {
                mutor.registers[reg as usize]
            } else {
                mutor.registers[REG_F as usize] |= FLAG_BIT_E;
                0
            }
        }
    }
    #[inline]
    pub(crate) fn write_reg(&self, reg: u8, val: u32) {
        unsafe { 
            let mutor = self.mutator();
            if reg < NUM_REGS as u8 {
                mutor.registers[reg as usize] = val;
            } else {
                mutor.registers[REG_F as usize] |= FLAG_BIT_E;
            }
        }
    }
    #[inline]
    pub(crate) fn read_reg_unchecked(&self, reg: u8) -> u32 {
        self.registers[reg as usize]
    }
    #[inline]
    pub(crate) fn write_reg_unchecked(&self, reg: u8, val: u32) {
        unsafe { self.mutator().registers[reg as usize] = val; }
    }
    #[inline]
    pub(crate) fn advance_ip(&self) {
        unsafe { self.mutator().registers[REG_I as usize] += 4; }
    }
    #[inline]
    pub(crate) fn split_instr(instr: u32) -> (u32, u8, u8, u8) {
        (instr >> 21 & 0b0000_0111_1111_1111, (instr >> 14 & 0b0111_1111) as u8, (instr >> 7 & 0b0111_1111) as u8, (instr & 0b0111_1111) as u8)
    }

    #[inline]
    pub unsafe fn mutator(&self) -> &mut Self {
        #[allow(mutable_transmutes)]
        std::mem::transmute(self)
    }
}

impl Drop for ThreadCore {
    fn drop(&mut self) {
        self.state.store(2, Ordering::Release);
        while self.state.load(Ordering::Relaxed) != 3 { std::thread::yield_now() }
        self.machine.thread_count.fetch_sub(1, Ordering::SeqCst);
    }
}