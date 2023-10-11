use crystalvm::{Machine, assemble};

fn main() {
    assemble("examples/hello_world.casm", "examples/hello_world.cstl").unwrap();
    let mut machine = Machine::from_image("examples/hello_world.cstl", 2u32.pow(16));
    println!("Running machine:");
    machine.run();
    loop {}
}
