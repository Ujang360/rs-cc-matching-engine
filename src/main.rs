use std::mem::{align_of, size_of};

mod orderbooks;

pub use orderbooks::*;

fn main() {
    println!("\n[OrderEvent]");
    println!("Packed Size        : {}", size_of::<OrderEvent>());
    println!("Alignment Size     : {}", align_of::<OrderEvent>());
    println!("\n[OrderMessage]");
    println!("Packed Size        : {}", size_of::<OrderMessage>());
    println!("Alignment Size     : {}", align_of::<OrderMessage>());
    println!("\n[OrderbookOrder]");
    println!("Packed Size        : {}", size_of::<OrderbookOrder>());
    println!("Alignment Size     : {}", align_of::<OrderbookOrder>());
    println!("\n[Orderbook]");
    println!("Packed Size        : {}", size_of::<Orderbook>());
    println!("Alignment Size     : {}", align_of::<Orderbook>());
    println!("\n[Orderbooks]");
    println!("Packed Size        : {}", size_of::<Orderbooks>());
    println!("Alignment Size     : {}", align_of::<Orderbooks>());
}
