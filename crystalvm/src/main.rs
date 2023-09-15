use crystalvm::Machine;



fn main() {
    let mut machine = Machine::run_from_image("../examples/hello_world.cstl", 0x10_000);
}
