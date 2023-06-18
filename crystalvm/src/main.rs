#![feature(seek_stream_len)]
#![feature(bigint_helper_methods)]


use machine::Machine;

use crate::machine::{REG_I};

pub mod machine;
pub mod screen;
pub mod device;

fn main() {
    let mut machine = Machine::from_image("../doodle.cstl", 0x8F000);
    loop {
        machine.execute_next();
    }
}
