#![feature(try_blocks)]
#![feature(seek_stream_len)]
#![feature(bigint_helper_methods)]
#![feature(buf_read_has_data_left)]


use debugger::Debugger;
use machine::Machine;



pub mod machine;
pub mod screen;
pub mod device;
pub mod debugger;

fn main() {
    let mut machine = Machine::from_image("../color.cstl", 0x90000, "Crystal VM", 3);
    let mut debugger = Debugger::with_debug_info_and_source(&mut machine, "../color.cdbg", "../color.casm");
    debugger.run()
}
