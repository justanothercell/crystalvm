use crystalvm::Machine;



fn main() {
    let mut machine = Machine::from_image("../examples/hello_world.cstl", 2u32.pow(16));
}
