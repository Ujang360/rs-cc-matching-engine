use chrono::Utc;
use std::mem;
use uuid::Uuid;

pub type OrderId = Uuid;
pub type OrderVolume = u64;
pub type OrderPrice = u64;
pub type OrderQuote = u64;
pub type UTCNanoSeconds = i64;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OrderSide {
    NoSide = 0,
    Bid = 1,
    Ask = 2,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OrderType {
    Cancel = 0,
    Market = 1,
    Limit = 2,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OrderEventType {
    NoMatch = 0,
    HasMatch = 1,
    Open = 2,
    Closed = 3,
    Cancelled = 4,
}

#[derive(Clone, Debug)]
#[repr(align(8))] // Packed Size is 64 bytes
pub struct OrderEvent {
    pub timestamp: UTCNanoSeconds,             // 8 bytes
    pub r#type: OrderEventType,                // 1 byte
    pub remaining_volume: Option<OrderVolume>, // 16 bytes
    pub traded_price: Option<OrderPrice>,      // 16 bytes
    pub crossed_id: Option<Uuid>,              // 16 bytes
}

#[derive(Clone, Debug)]
#[repr(align(8))] // Packed Size is 96 bytes
pub struct OrderMessage {
    pub id: OrderId,                   // 16 bytes
    pub target_id: Option<OrderId>,    // 16 bytes
    pub created_at: UTCNanoSeconds,    // 8 bytes
    pub side: OrderSide,               // 1 byte
    pub r#type: OrderType,             // 1 byte
    pub volume: Option<OrderVolume>,   // 16 bytes
    pub price: Option<OrderPrice>,     // 16 bytes
    pub max_quote: Option<OrderQuote>, // 16 bytes
    pub events: Vec<OrderEvent>,       // 24 bytes
}

#[derive(Clone, Debug)]
#[repr(align(8))] // Packed Size 24 bytes
pub struct OrderbookOrder {
    pub id: OrderId,                   // 16 bytes
    pub remaining_volume: OrderVolume, // 8 bytes
}

impl Default for OrderMessage {
    fn default() -> OrderMessage {
        OrderMessage {
            id: Uuid::new_v4(),
            target_id: None,
            created_at: Utc::now().timestamp_nanos(),
            side: OrderSide::NoSide,
            r#type: OrderType::Limit,
            volume: None,
            price: None,
            max_quote: None,
            events: Vec::new(),
        }
    }
}

impl PartialEq for OrderSide {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl PartialEq for OrderType {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl PartialEq for OrderEventType {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl Eq for OrderSide {}
impl Eq for OrderType {}
impl Eq for OrderEventType {}
