use std::{sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering, AtomicU8}}, collections::HashMap};

use crate::machine::MachineCtx;

pub struct ThreadCore {
    pub machine: Arc<MachineCtx>,
    pub children: HashMap<u32, Arc<ThreadCore>>,

    // ready: 0, running: 1, terminating: 2, terminated: 3
    pub state: AtomicU8,
    pub access_min_addr: u32,
    pub access_max_addr: u32,
    pub permissions: AtomicU32,

    pub thread_id: u32,
    pub parent_thread_id: u32
}

impl ThreadCore {
    pub fn is_child_of(&self, tid: u32) -> bool {
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

    pub fn is_parent_of(&self, tid: u32) -> bool {
        match self.machine.threads.get(&tid) {
            Some(t) => t.is_child_of(self.thread_id),
            None => return false
        }
    }

    pub fn start(self: Arc<Self>) {
        std::thread::spawn(move || {
            self.state.store(1, Ordering::Release);
            self.run();
            self.state.store(3, Ordering::Release);
        });
    }
    pub fn run(&self) {
        loop {
            // request quit?
            if self.state.load(Ordering::Relaxed) == 2 { return; }
        }
    }
    #[inline]
    pub fn atomic_op<F: FnOnce(u32) -> u32>(&self, addr: u32, f: F) {
        while self.machine.atomic_lock.swap(true, Ordering::SeqCst) { std::thread::yield_now() }
        f(0);
        self.machine.atomic_lock.store(false, Ordering::SeqCst);
    }
    #[inline]
    pub fn read_u8(&self, addr: u32) -> Result<u8, ()> {
        if addr >= self.access_min_addr && addr <= self.access_max_addr {
            Ok(self.machine.memory[addr as usize])
        } else {
            //TODO: set err flag
            Err(())
        }
    }
    #[inline]
    pub fn write_u8(&self, addr: u32, value: u8) -> Result<(), ()> {
        if addr >= self.access_min_addr && addr <= self.access_max_addr {
            self.machine.mem_mut()[addr as usize] = value;
            Ok(())
        } else {
            //TODO: set err flag
            Err(())
        }
    }
    #[inline]
    pub fn read_u32(&self, addr: u32) -> Result<u32, ()> {
        if addr >= self.access_min_addr && addr + 3 <= self.access_max_addr {
            Ok(u32::from_le_bytes(self.machine.memory[addr as usize .. addr as usize + 4].try_into().unwrap()))
        } else {
            //TODO: set err flag
            Err(())
        }
    }
    #[inline]
    pub fn write_u32(&self, addr: u32, value: u32) -> Result<(), ()> {
        if addr >= self.access_min_addr && addr + 3 <= self.access_max_addr {
            unsafe { std::ptr::copy_nonoverlapping(&mut value as *mut u32 as *mut _, (self.machine.mem_mut() as usize + addr as usize) as *const u8, std::mem::size_of::<u32>()); }
            Ok(())
        } else {
            //TODO: set err flag
            Err(())
        }
    }

    #[inline]
    pub unsafe fn mutator(&self) -> &mut Self{

    }
}

impl Drop for ThreadCore {
    fn drop(&mut self) {
        self.state.store(2, Ordering::Release);
        while self.state.load(Ordering::Relaxed) != 3 { std::thread::yield_now() }
        self.machine.thread_count.fetch_sub(1, Ordering::SeqCst);
    }
}