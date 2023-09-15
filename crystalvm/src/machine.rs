use std::{path::Path, fs::File, io::{Seek, Read}, collections::HashMap, sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering}}};

use crate::thread::ThreadCore;

pub struct Machine {
    pub memory: Box<Vec<u8>>,
    pub running: Arc<AtomicBool>,
    pub threads: HashMap<u32, ThreadCore>,
    pub thread_count: Arc<AtomicU32>,
    pub next_thead_id: Arc<AtomicU32>,
}

impl Machine {
    pub fn run_from_image<P: AsRef<Path>>(path: P, memory_size: usize) -> Self {
        let mut image = File::open(path).unwrap();
        let img_size = image.stream_len().unwrap() as usize;
        let mut image_contents = Vec::with_capacity(img_size);
        image.read_to_end(&mut image_contents).unwrap();
        if memory_size < img_size {
            panic!("need at least 0x{:X} bytes, only got 0x{:X} supplied", img_size, memory_size)
        }
        let mut memory = Box::new(Vec::with_capacity(memory_size));
        // actually zero initialize it
        for _ in 0..memory_size {
            memory.push(0);
        }
        unsafe{ 
            std::ptr::copy_nonoverlapping(image_contents.as_ptr(), memory.as_mut_ptr(), image_contents.len());
        }
        let running = Arc::new(AtomicBool::new(true));
        Machine {  
            memory
        }
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        while self.thread_count.load(Ordering::Relaxed) > 0 { std::thread::yield_now() }
    }
}