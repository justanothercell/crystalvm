mod instructions;

use std::{sync::{Arc, atomic::{Ordering, AtomicU8}}, collections::HashMap};

use crate::machine::MachineCtx;

/// Instruction Pointer
pub const REG_I: u32 = 0x30;
/// Frame Pointer
pub const REG_W: u32 = 0x31;
/// Stack Pointer
pub const REG_S: u32 = 0x32;
/// Interrupt Table
pub const REG_T: u32 = 0x33;
/// Flags Register
pub const REG_F: u32 = 0x34;
/// Interrupting Device ID
pub const REG_D: u32 = 0x35;
/// Carry/Overflow Register
pub const REG_C: u32 = 0x35;

pub const NUM_REGS: u32 = 0x36;

// Flags
/// zero: Z = a != b
pub const FLAG_BIT_Z: u32 = 1 << 0;
/// sign: S = a < b
pub const FLAG_BIT_S: u32 = 1 << 1;
/// carry: an operation over or underflowed/produced a carry value
pub const FLAG_BIT_C: u32 = 1 << 2;


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
    fn is_child_of(&self, tid: u32) -> bool {
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
        }
    }

    #[inline]
    fn atomic_op<F: FnOnce(u32) -> u32>(&self, addr: u32, f: F) {
        while self.machine.atomic_lock.swap(true, Ordering::SeqCst) { std::thread::yield_now() }
        f(0);
        self.machine.atomic_lock.store(false, Ordering::SeqCst);
    }
    #[inline]
    fn read_u8(&self, addr: u32) -> Result<u8, ()> {
        if addr >= self.access_min_addr && addr <= self.access_max_addr {
            Ok(self.machine.memory[addr as usize])
        } else {
            //TODO: set err flag
            Err(())
        }
    }
    #[inline]
    fn write_u8(&self, addr: u32, value: u8) -> Result<(), ()> {
        if addr >= self.access_min_addr && addr <= self.access_max_addr {
            self.machine.mem_mut()[addr as usize] = value;
            Ok(())
        } else {
            //TODO: set err flag
            Err(())
        }
    }
    #[inline]
    fn read_u32(&self, addr: u32) -> Result<u32, ()> {
        if addr >= self.access_min_addr && addr + 3 <= self.access_max_addr {
            Ok(u32::from_le_bytes(self.machine.memory[addr as usize .. addr as usize + 4].try_into().unwrap()))
        } else {
            //TODO: set err flag
            Err(())
        }
    }
    #[inline]
    fn write_u32(&self, addr: u32, value: u32) -> Result<(), ()> {
        if addr >= self.access_min_addr && addr + 3 <= self.access_max_addr {
            unsafe { std::ptr::copy_nonoverlapping(&value as *const u32 as *mut _, (self.machine.mem_mut() as *mut _ as usize + addr as usize) as *mut u8, std::mem::size_of::<u32>()); }
            Ok(())
        } else {
            //TODO: set err flag
            Err(())
        }
    }

    #[inline]
    pub unsafe fn mutator(&self) -> &mut Self {
        &mut *(self as *const _ as *mut _)
    }
}

impl Drop for ThreadCore {
    fn drop(&mut self) {
        self.state.store(2, Ordering::Release);
        while self.state.load(Ordering::Relaxed) != 3 { std::thread::yield_now() }
        self.machine.thread_count.fetch_sub(1, Ordering::SeqCst);
    }
}