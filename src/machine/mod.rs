pub(crate) mod thread;
pub(crate) mod device;

use std::{path::Path, fs::File, io::{Seek, Read}, collections::HashMap, sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering}}};

use self::thread::ThreadCore;


pub struct Machine {
    pub ctx: Arc<MachineCtx>
}

pub struct MachineCtx {
    pub memory: Box<Vec<u8>>,

    pub threads: HashMap<u32, Arc<ThreadCore>>,

    pub running: AtomicBool,
    pub atomic_lock: AtomicBool,
    pub thread_count: AtomicU32,
    pub next_thead_id: AtomicU32,
}

impl MachineCtx {
    #[inline]
    pub fn mem_mut<'a>(&'a self) -> &'a mut Vec<u8> {
        #[allow(mutable_transmutes)]
        #[allow(invalid_reference_casting)]
        unsafe { &mut *(&*self.memory as *const _ as *mut _) }
    }
    #[inline]
    pub(crate) unsafe fn mutator(&self) -> &mut Self {
        #[allow(mutable_transmutes)]
        #[allow(invalid_reference_casting)]
        &mut *(self as *const _ as *mut _)
    }
}

impl Machine {
    pub fn from_image<P: AsRef<Path>>(path: P, memory_size: u32) -> Self {
        let mut image = File::open(path).unwrap();
        let img_size = image.stream_len().unwrap() as usize;
        if memory_size < img_size as u32 {
            panic!("need at least 0x{:X} bytes, only got 0x{:X} supplied", img_size, memory_size)
        }
        let mut memory = Box::new(Vec::with_capacity(memory_size as usize));
        image.read_to_end(&mut memory).unwrap();
        // zero initialize the rest
        for _ in memory.len()..memory_size as usize {
            memory.push(0);
        }
        let ctx = Arc::new(MachineCtx { 
            memory, 
            threads: Default::default(),
            running: AtomicBool::new(true), 
            thread_count: AtomicU32::new(0), 
            next_thead_id: AtomicU32::new(0),
            atomic_lock: AtomicBool::new(false),
        });
        Machine {
            ctx
        }
    }

    pub fn run(mut self) {
        ThreadCore::launch_main(&mut self);
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        self.ctx.running.store(false, Ordering::Release);
        while self.ctx.thread_count.load(Ordering::Relaxed) > 0 { std::thread::yield_now() }
    }
}