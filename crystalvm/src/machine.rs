use std::{path::Path, fs::File, io::{Seek, Read}, collections::HashMap, sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering}}};

use crate::thread::ThreadCore;

pub struct Machine {
    pub memory: Box<Vec<u8>>,
    pub ctx: Arc<MachineCtx>
}

pub struct MachineCtx {
    pub memory: &'static Vec<u8>,

    pub threads: HashMap<u32, ThreadCore>,

    pub running: AtomicBool,
    pub atomic_lock: AtomicBool,
    pub thread_count: AtomicU32,
    pub next_thead_id: AtomicU32,
}

impl MachineCtx {
    #[inline]
    pub fn mem_mut<'a>(&'a self) -> &'a mut Vec<u8> {
        unsafe { &mut *(self.memory as *const _ as *mut _) }
    }
}

impl Machine {
    pub fn from_image<P: AsRef<Path>>(path: P, memory_size: u32) -> Self {
        let mut image = File::open(path).unwrap();
        let img_size = image.stream_len().unwrap() as usize;
        let mut image_contents = Vec::with_capacity(img_size);
        image.read_to_end(&mut image_contents).unwrap();
        if memory_size < img_size as u32 {
            panic!("need at least 0x{:X} bytes, only got 0x{:X} supplied", img_size, memory_size)
        }
        let mut memory = Box::new(Vec::with_capacity(memory_size as usize));
        // actually zero initialize it
        for _ in 0..memory_size {
            memory.push(0);
        }
        unsafe{ 
            std::ptr::copy_nonoverlapping(image_contents.as_ptr(), memory.as_mut_ptr(), image_contents.len());
        }
        let ctx = Arc::new(MachineCtx { 
            memory: unsafe { &*(memory.as_ref() as *const _) }, 
            threads: Default::default(),
            running: AtomicBool::new(true), 
            thread_count: AtomicU32::new(0), 
            next_thead_id: AtomicU32::new(0),
            atomic_lock: AtomicBool::new(false),
        });
        Machine {  
            memory,
            ctx
        }
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        self.ctx.running.store(false, Ordering::Release);
        while self.ctx.thread_count.load(Ordering::Relaxed) > 0 { std::thread::yield_now() }
    }
}