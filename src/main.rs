use chrono::prelude::*;
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

fn bench_perfect_limit_match(match_count: u64) {
    println!(
        "\n[Benchmark: {} Limit Match ({} Orders)]",
        match_count,
        match_count * 2
    );
    print!("- Populating Orders...");
    let mut the_orderbooks = Orderbooks::default();
    let mut limit_bid_orders = Vec::new();
    let mut limit_ask_orders = Vec::new();
    for i in 0..match_count {
        limit_bid_orders.push(OrderMessage {
            r#type: OrderType::Limit,
            side: OrderSide::Bid,
            volume: Some(20_000),
            price: Some(100_000 - (i % 1000)),
            ..Default::default()
        });
    }
    for i in 0..match_count {
        limit_ask_orders.push(OrderMessage {
            r#type: OrderType::Limit,
            side: OrderSide::Ask,
            volume: Some(20_000),
            price: Some(90_000 + (i % 1000)),
            ..Default::default()
        });
    }
    println!("DONE");
    print!("- Matching...");
    let timestamp_start = Utc::now().timestamp_nanos();
    for limit_bid_order in limit_bid_orders {
        the_orderbooks.execute_order(&limit_bid_order);
    }
    for limit_ask_order in limit_ask_orders {
        the_orderbooks.execute_order(&limit_ask_order);
    }
    let timestamp_end = Utc::now().timestamp_nanos();
    println!("DONE");
    let exec_span_nano = timestamp_end - timestamp_start;
    let ops = 1_000_000_000 / (exec_span_nano as u64 / (match_count * 2));
    println!("- Took {} ns to complete", exec_span_nano);
    println!("- {} Orders per second", ops);
    let (bids_count, asks_count, _) = the_orderbooks.count();
    println!("- Orderbook Bids count {} orders", bids_count);
    println!("- Orderbook Asks count {} orders", asks_count);
}

fn main() {
    print_info_headers();
    print_structure_info();
    bench_perfect_limit_match(5);
    bench_perfect_limit_match(50);
    bench_perfect_limit_match(500);
    bench_perfect_limit_match(5_000);
    bench_perfect_limit_match(50_000);
    bench_perfect_limit_match(500_000);
}
