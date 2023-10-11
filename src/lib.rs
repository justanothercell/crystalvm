#![feature(io_error_more)]
#![feature(seek_stream_len)]
#![feature(bigint_helper_methods)]
#![feature(try_blocks)]
#![feature(macro_metavar_expr)]


pub(crate) mod machine;
mod assembler;

pub use machine::Machine;
pub use assembler::assemble;