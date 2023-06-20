#![feature(seek_stream_len)]
#![feature(bigint_helper_methods)]


use machine::Machine;



pub mod machine;
pub mod screen;
pub mod device;

fn main() {
    let mut machine = Machine::from_image("../color.cstl", 0x8F000, "Crystal VM", 3);
    loop {
        machine.execute_next();
    }
}
