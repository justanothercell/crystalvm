use crystalvm::{Debugger, Machine};


fn main() {
    let mut machine = Machine::from_image("../examples/keyboard_cheap.cstl", 0x90000, "Crystal VM", 3);
    let kbd = machine.take_keyboard().unwrap();
    machine.attach_device(kbd);
    //let mut debugger = Debugger::with_debug_info_and_source(&mut machine, "../examples/keyboard_cheap.cdbg", "../examples/keyboard_cheap.casm");
    //debugger.run();
    loop { machine.execute_next() }
}
