use std::mem::{align_of, size_of};

mod orderbooks;

pub use orderbooks::*;

macro_rules! show_size {
    ($t:ty) => {
        println!(
            "- {}:\n  * Size {} bytes\n  * Aligment {} bytes",
            stringify!($t),
            size_of::<$t>(),
            align_of::<$t>()
        );
    };
}

fn print_info_headers() {
    println!();
    println!("CC Matching Engine (Rust) - 1.0.0-beta.0");
    println!("========================================");
    println!();
}

fn print_structure_info() {
    println!("[Data Structure Alignment]");
    show_size!(OrderEvent);
    show_size!(OrderMessage);
    show_size!(OrderbookOrder);
    show_size!(Orderbook);
    show_size!(Orderbooks);
}

fn main() {
    print_info_headers();
    print_structure_info();
}
