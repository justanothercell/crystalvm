use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering, AtomicU8}};

pub struct ThreadCore {
    pub memory: &'static mut Vec<u8>,
    pub machine_running: Arc<AtomicBool>,
    // ready: 0, running: 1, terminating: 2, terminated: 3
    pub state: Arc<AtomicU8>,
    pub thread_count: Arc<AtomicU32>,
    pub next_thead_id: Arc<AtomicU32>,
    pub thread_id: u32,
    pub access_min_addr: u32,
    pub access_max_addr: u32,
    pub permissions: u32
}

impl ThreadCore {
    pub fn fork(&mut self) -> ThreadCore {
        let tid = self.next_thead_id.fetch_add(1, Ordering::SeqCst);
        ThreadCore {
            memory: self.memory,
            machine_running: self.machine_running.clone(), 
            thread_count: self.thread_count.clone(), 
            next_thead_id: self.next_thead_id.clone(), 
            thread_id: tid,
            access_min_addr: self.access_min_addr,
            access_max_addr: self.access_max_addr,
            permissions: self.permissions
        }
    }
}

impl Drop for ThreadCore {
    fn drop(&mut self) {
        self.thread_count.fetch_sub(1, Ordering::SeqCst);
    }
}