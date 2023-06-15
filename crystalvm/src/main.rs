#![feature(seek_stream_len)]
#![feature(get_many_mut)]


use machine::Machine;

pub mod machine;
pub mod screen;
pub mod device;

fn main() {
    let mut machine = Machine::from_image("../doodle.cstl", 0x22000 + 0x1000);

    loop {
        machine.execute_next();
        println!("{:08X?}", [
            machine.registers[0], machine.registers[1], machine.registers[2], machine.registers[3], 
            machine.registers[48], machine.registers[49], machine.registers[50], machine.registers[51], machine.registers[52]
        ])
    }
}
