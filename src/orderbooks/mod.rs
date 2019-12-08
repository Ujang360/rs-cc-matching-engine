mod orders;

pub use orders::*;

use chrono::prelude::*;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub type PriceLevel = u64;
pub type Index = usize;

#[derive(Clone, Debug)]
#[repr(align(8))]
pub struct Orderbook {
    pub orders: BTreeMap<PriceLevel, Vec<OrderbookOrder>>,
    pub orders_location: HashMap<Uuid, (PriceLevel, Index)>,
}

#[derive(Clone, Debug)]
#[repr(align(8))]
pub struct Orderbooks {
    pub bids: Orderbook,
    pub asks: Orderbook,
    pub orders_location: HashMap<Uuid, OrderSide>,
}

impl Orderbook {
    pub fn count(&self) -> usize {
        self.orders_location.len()
    }

    pub fn remove(&mut self, order_id: &Uuid) -> Option<OrderbookOrder> {
        if !self.orders_location.contains_key(order_id) {
            return None;
        }

        let mut retval = None;
        let (_, (price_level, index)) = self.orders_location.remove_entry(order_id).unwrap();

        if let Some(price_level_orders) = self.orders.get_mut(&price_level) {
            retval = Some(price_level_orders.remove(index));

            if price_level_orders.is_empty() {
                self.orders.remove(&price_level);
            }
        }

        retval
    }

    pub fn insert(&mut self, price_level: OrderPrice, order: OrderbookOrder) {
        if self.orders_location.contains_key(&order.id) {
            panic!("Trying to insert an already inserted order: {}", order.id);
        }

        let order_id = order.id;
        let mut insertion_index = 0;

        if let Some(pricelevel_orders) = self.orders.get_mut(&price_level) {
            insertion_index = pricelevel_orders.len();
            pricelevel_orders.push(order)
        } else {
            self.orders.insert(price_level, [order].to_vec());
        }

        self.orders_location.insert(order_id, (price_level, insertion_index));
    }
}

impl Orderbooks {
    pub fn count(&self) -> (usize, usize, usize) {
        let bids_count = self.bids.count();
        let asks_count = self.asks.count();

        (bids_count, asks_count, bids_count + asks_count)
    }

    pub fn remove(&mut self, order_id: &Uuid) -> Option<OrderbookOrder> {
        if !self.orders_location.contains_key(order_id) {
            return None;
        }

        let (_, orderbook_side) = self.orders_location.remove_entry(order_id).unwrap();

        match orderbook_side {
            OrderSide::Bid => self.bids.remove(order_id),
            OrderSide::Ask => self.asks.remove(order_id),
        }
    }

    pub fn insert(&mut self, order_message: &OrderMessage, remaining_volume: OrderVolume) {
        if self.orders_location.contains_key(&order_message.id) {
            panic!("Trying to insert an already inserted order: {}", order_message.id);
        }

        let order_id = order_message.id;

        if order_message.r#type == OrderType::Cancel {
            panic!("Trying to insert a cancellation order: {}", order_id);
        }

        let orderbook = match order_message.side {
            OrderSide::Bid => &mut self.bids,
            OrderSide::Ask => &mut self.asks,
        };
        let new_orderbook_order = OrderbookOrder {
            id: order_id,
            remaining_volume,
        };
        let order_price = order_message.price.unwrap();

        orderbook.insert(order_price, new_orderbook_order);
        self.orders_location.insert(order_id, order_message.side);
    }

    pub fn execute_order(&mut self, order_message: &OrderMessage) -> HashMap<Uuid, Vec<OrderEvent>> {
        let current_order_id = order_message.id;
        let current_order_type = order_message.r#type;
        let current_order_side = order_message.side;
        let current_order_max_quote = order_message.max_quote;
        let current_order_volume = order_message.volume;
        let current_order_price = order_message.price;
        let mut current_order_events = Vec::new();
        let mut order_events = HashMap::new();
        let current_order_events_ref = &mut current_order_events;
        let order_events_ref = &mut order_events;
        let mut order_remaining_volume = current_order_volume.unwrap();
        let mut order_traded_volume = 0;
        let mut pending_order_removal_id = Vec::new();

        match current_order_type {
            OrderType::Cancel => {
                let removed_order = self.remove(&order_message.target_id.unwrap()).unwrap();
                let current_timestamp = Utc::now().timestamp_nanos();
                let original_order_event = OrderEvent {
                    timestamp: current_timestamp,
                    r#type: OrderEventType::Cancelled,
                    remaining_volume: Some(removed_order.remaining_volume),
                    crossed_id: Some(removed_order.id),
                    traded_price: None,
                };
                let cancel_order_event = OrderEvent {
                    timestamp: current_timestamp,
                    r#type: OrderEventType::Closed,
                    remaining_volume: None,
                    crossed_id: Some(current_order_id),
                    traded_price: None,
                };
                current_order_events_ref.push(cancel_order_event);
                order_events_ref.insert(removed_order.id, [original_order_event].to_vec());
            }
            OrderType::Market => {
                match current_order_side {
                    OrderSide::Ask => {
                        if !self.bids.orders_location.is_empty() {
                            let bid_orderbook_iter = self.bids.orders.iter_mut().rev();

                            for (price_level_ref, next_bid_orders_ref) in bid_orderbook_iter {
                                let price_level = *price_level_ref;
                                let traded_price = Some(price_level);
                                let pricelevel_bid_orders_iter = next_bid_orders_ref.iter_mut();

                                for next_bid_order_ref in pricelevel_bid_orders_iter {
                                    let bid_order_id = next_bid_order_ref.id;

                                    if next_bid_order_ref.remaining_volume >= order_remaining_volume {
                                        order_traded_volume += order_remaining_volume;
                                        next_bid_order_ref.remaining_volume -= order_remaining_volume;
                                        order_remaining_volume = 0;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(bid_order_id),
                                            traded_price,
                                        });
                                        let mut bid_order_events = Vec::new();
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_bid_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });

                                        if next_bid_order_ref.remaining_volume == 0 {
                                            bid_order_events.push(OrderEvent {
                                                timestamp: Utc::now().timestamp_nanos(),
                                                r#type: OrderEventType::Closed,
                                                remaining_volume: Some(0),
                                                crossed_id: None,
                                                traded_price: None,
                                            });
                                            pending_order_removal_id.push(bid_order_id);
                                        }

                                        order_events_ref.insert(bid_order_id, bid_order_events);
                                        break;
                                    } else {
                                        let bid_order_traded_volume = next_bid_order_ref.remaining_volume;
                                        next_bid_order_ref.remaining_volume = 0;
                                        order_remaining_volume -= bid_order_traded_volume;
                                        order_traded_volume += bid_order_traded_volume;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(bid_order_id),
                                            traded_price,
                                        });
                                        let mut bid_order_events = Vec::new();
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_bid_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::Closed,
                                            remaining_volume: Some(0),
                                            crossed_id: None,
                                            traded_price: None,
                                        });
                                        pending_order_removal_id.push(bid_order_id);
                                        order_events_ref.insert(bid_order_id, bid_order_events);
                                    }

                                    if order_remaining_volume == 0 {
                                        break;
                                    }
                                }

                                if order_remaining_volume == 0 {
                                    break;
                                }
                            }

                            while let Some(pending_removal_bid_id) = pending_order_removal_id.pop() {
                                self.bids.remove(&pending_removal_bid_id);
                            }
                        }
                    }
                    OrderSide::Bid => {
                        if !self.asks.orders_location.is_empty() {
                            let mut order_remaining_quote = current_order_max_quote.unwrap();
                            let ask_orderbook_iter = self.asks.orders.iter_mut();

                            for (price_level_ref, next_ask_orders_ref) in ask_orderbook_iter {
                                let price_level = *price_level_ref;
                                let traded_price = Some(price_level);
                                let pricelevel_ask_orders_iter = next_ask_orders_ref.iter_mut();

                                if order_remaining_quote < price_level {
                                    break;
                                }

                                let mut remaining_pricelevel_volume = order_remaining_quote / price_level;

                                for next_ask_order_ref in pricelevel_ask_orders_iter {
                                    let ask_order_id = next_ask_order_ref.id;

                                    if next_ask_order_ref.remaining_volume >= remaining_pricelevel_volume {
                                        order_traded_volume += remaining_pricelevel_volume;
                                        next_ask_order_ref.remaining_volume -= remaining_pricelevel_volume;
                                        order_remaining_volume -= remaining_pricelevel_volume;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(ask_order_id),
                                            traded_price,
                                        });
                                        let mut ask_order_events = Vec::new();
                                        ask_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_ask_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });

                                        if next_ask_order_ref.remaining_volume == 0 {
                                            ask_order_events.push(OrderEvent {
                                                timestamp: Utc::now().timestamp_nanos(),
                                                r#type: OrderEventType::Closed,
                                                remaining_volume: Some(0),
                                                crossed_id: None,
                                                traded_price: None,
                                            });
                                            pending_order_removal_id.push(ask_order_id);
                                        }

                                        order_events_ref.insert(ask_order_id, ask_order_events);
                                        break;
                                    } else {
                                        let ask_order_traded_volume = next_ask_order_ref.remaining_volume;
                                        next_ask_order_ref.remaining_volume = 0;
                                        order_remaining_volume -= ask_order_traded_volume;
                                        order_traded_volume += ask_order_traded_volume;
                                        remaining_pricelevel_volume -= ask_order_traded_volume;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(ask_order_id),
                                            traded_price,
                                        });
                                        let mut ask_order_events = Vec::new();
                                        ask_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_ask_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });
                                        ask_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::Closed,
                                            remaining_volume: Some(0),
                                            crossed_id: None,
                                            traded_price: None,
                                        });
                                        pending_order_removal_id.push(ask_order_id);
                                        order_events_ref.insert(ask_order_id, ask_order_events);
                                    }

                                    if order_remaining_volume == 0 {
                                        break;
                                    }
                                }

                                order_remaining_quote -= order_traded_volume * price_level;
                                if order_remaining_volume == 0 {
                                    break;
                                }
                            }

                            while let Some(pending_removal_ask_id) = pending_order_removal_id.pop() {
                                self.asks.remove(&pending_removal_ask_id);
                            }
                        }
                    }
                }

                if order_traded_volume == 0 {
                    current_order_events_ref.push(OrderEvent {
                        timestamp: Utc::now().timestamp_nanos(),
                        r#type: OrderEventType::NoMatch,
                        remaining_volume: Some(order_remaining_volume),
                        crossed_id: None,
                        traded_price: None,
                    })
                }

                current_order_events_ref.push(OrderEvent {
                    timestamp: Utc::now().timestamp_nanos(),
                    r#type: OrderEventType::Closed,
                    remaining_volume: Some(order_remaining_volume),
                    crossed_id: None,
                    traded_price: None,
                });
            }
            OrderType::Limit => {
                let order_price = current_order_price.unwrap();

                match current_order_side {
                    OrderSide::Ask => {
                        if self.bids.orders_location.is_empty() {
                            self.asks.insert(
                                order_price,
                                OrderbookOrder {
                                    id: current_order_id,
                                    remaining_volume: order_remaining_volume,
                                },
                            );
                        } else {
                            let bid_orderbook_iter = self.bids.orders.iter_mut().rev();

                            for (price_level_ref, next_bid_orders_ref) in bid_orderbook_iter {
                                let price_level = *price_level_ref;

                                if price_level < order_price {
                                    break;
                                }

                                let traded_price = Some(price_level);
                                let pricelevel_bid_orders_iter = next_bid_orders_ref.iter_mut();

                                for next_bid_order_ref in pricelevel_bid_orders_iter {
                                    let bid_order_id = next_bid_order_ref.id;

                                    if next_bid_order_ref.remaining_volume >= order_remaining_volume {
                                        order_traded_volume += order_remaining_volume;
                                        next_bid_order_ref.remaining_volume -= order_remaining_volume;
                                        order_remaining_volume = 0;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(bid_order_id),
                                            traded_price,
                                        });
                                        let mut bid_order_events = Vec::new();
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_bid_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });

                                        if next_bid_order_ref.remaining_volume == 0 {
                                            bid_order_events.push(OrderEvent {
                                                timestamp: Utc::now().timestamp_nanos(),
                                                r#type: OrderEventType::Closed,
                                                remaining_volume: Some(0),
                                                crossed_id: None,
                                                traded_price: None,
                                            });
                                            pending_order_removal_id.push(bid_order_id);
                                        }

                                        order_events_ref.insert(bid_order_id, bid_order_events);
                                        break;
                                    } else {
                                        let bid_order_traded_volume = next_bid_order_ref.remaining_volume;
                                        next_bid_order_ref.remaining_volume = 0;
                                        order_remaining_volume -= bid_order_traded_volume;
                                        order_traded_volume += bid_order_traded_volume;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(bid_order_id),
                                            traded_price,
                                        });
                                        let mut bid_order_events = Vec::new();
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_bid_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::Closed,
                                            remaining_volume: Some(0),
                                            crossed_id: None,
                                            traded_price: None,
                                        });
                                        pending_order_removal_id.push(bid_order_id);
                                        order_events_ref.insert(bid_order_id, bid_order_events);
                                    }

                                    if order_remaining_volume == 0 {
                                        break;
                                    }
                                }

                                if order_remaining_volume == 0 {
                                    break;
                                }
                            }

                            if order_remaining_volume > 0 {
                                self.asks.insert(
                                    order_price,
                                    OrderbookOrder {
                                        id: current_order_id,
                                        remaining_volume: order_remaining_volume,
                                    },
                                );
                            }

                            while let Some(pending_removal_bid_id) = pending_order_removal_id.pop() {
                                self.bids.remove(&pending_removal_bid_id);
                            }
                        }
                    }
                    OrderSide::Bid => {
                        if self.asks.orders_location.is_empty() {
                            self.bids.insert(
                                order_price,
                                OrderbookOrder {
                                    id: current_order_id,
                                    remaining_volume: order_remaining_volume,
                                },
                            );
                        } else {
                            let ask_orderbook_iter = self.asks.orders.iter_mut();

                            for (price_level_ref, next_ask_orders_ref) in ask_orderbook_iter {
                                let price_level = *price_level_ref;

                                if price_level > order_price {
                                    break;
                                }

                                let traded_price = Some(price_level);
                                let pricelevel_ask_orders_iter = next_ask_orders_ref.iter_mut();

                                for next_ask_order_ref in pricelevel_ask_orders_iter {
                                    let ask_order_id = next_ask_order_ref.id;

                                    if next_ask_order_ref.remaining_volume >= order_remaining_volume {
                                        order_traded_volume += order_remaining_volume;
                                        next_ask_order_ref.remaining_volume -= order_remaining_volume;
                                        order_remaining_volume = 0;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(ask_order_id),
                                            traded_price,
                                        });
                                        let mut bid_order_events = Vec::new();
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_ask_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });

                                        if next_ask_order_ref.remaining_volume == 0 {
                                            bid_order_events.push(OrderEvent {
                                                timestamp: Utc::now().timestamp_nanos(),
                                                r#type: OrderEventType::Closed,
                                                remaining_volume: Some(0),
                                                crossed_id: None,
                                                traded_price: None,
                                            });
                                            pending_order_removal_id.push(ask_order_id);
                                        }

                                        order_events_ref.insert(ask_order_id, bid_order_events);
                                        break;
                                    } else {
                                        let ask_order_traded_volume = next_ask_order_ref.remaining_volume;
                                        next_ask_order_ref.remaining_volume = 0;
                                        order_remaining_volume -= ask_order_traded_volume;
                                        order_traded_volume += ask_order_traded_volume;
                                        current_order_events_ref.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(order_remaining_volume),
                                            crossed_id: Some(ask_order_id),
                                            traded_price,
                                        });
                                        let mut bid_order_events = Vec::new();
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::HasMatch,
                                            remaining_volume: Some(next_ask_order_ref.remaining_volume),
                                            crossed_id: Some(current_order_id),
                                            traded_price,
                                        });
                                        bid_order_events.push(OrderEvent {
                                            timestamp: Utc::now().timestamp_nanos(),
                                            r#type: OrderEventType::Closed,
                                            remaining_volume: Some(0),
                                            crossed_id: None,
                                            traded_price: None,
                                        });
                                        pending_order_removal_id.push(ask_order_id);
                                        order_events_ref.insert(ask_order_id, bid_order_events);
                                    }

                                    if order_remaining_volume == 0 {
                                        break;
                                    }
                                }

                                if order_remaining_volume == 0 {
                                    break;
                                }
                            }

                            if order_remaining_volume > 0 {
                                self.bids.insert(
                                    order_price,
                                    OrderbookOrder {
                                        id: current_order_id,
                                        remaining_volume: order_remaining_volume,
                                    },
                                );
                            }

                            while let Some(pending_removal_ask_id) = pending_order_removal_id.pop() {
                                self.asks.remove(&pending_removal_ask_id);
                            }
                        }
                    }
                }

                if order_traded_volume == 0 {
                    current_order_events_ref.push(OrderEvent {
                        timestamp: Utc::now().timestamp_nanos(),
                        r#type: OrderEventType::NoMatch,
                        remaining_volume: Some(order_remaining_volume),
                        crossed_id: None,
                        traded_price: None,
                    })
                }

                if order_remaining_volume > 0 {
                    current_order_events_ref.push(OrderEvent {
                        timestamp: Utc::now().timestamp_nanos(),
                        r#type: OrderEventType::Open,
                        remaining_volume: Some(order_remaining_volume),
                        crossed_id: None,
                        traded_price: None,
                    })
                } else {
                    current_order_events_ref.push(OrderEvent {
                        timestamp: Utc::now().timestamp_nanos(),
                        r#type: OrderEventType::Closed,
                        remaining_volume: Some(0),
                        crossed_id: None,
                        traded_price: None,
                    })
                }
            }
        }

        order_events_ref.insert(current_order_id, current_order_events);

        order_events
    }
}
