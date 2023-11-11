use crystalvm::{Machine, assemble};

fn main() {
    assemble("examples/alloc_test.casm", "examples/alloc_test.cstl").unwrap();
    let machine = Machine::from_image("examples/alloc_test.cstl", 2u32.pow(16));
    println!("Running machine:");
    machine.run();
    loop {
        std::thread::yield_now()
    }
}
