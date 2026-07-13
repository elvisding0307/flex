use std::collections::{BTreeMap, VecDeque};

use crate::order::{Order, Side};

pub struct OrderBook {
    // Bids: sorted by price ascending. Best bid = highest price = last entry.
    bids: BTreeMap<u64, VecDeque<Order>>,
    // Asks: sorted by price ascending. Best ask = lowest price = first entry.
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, order: Order) {
        let book = match order.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };
        book.entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn best_bid_price(&self) -> Option<u64> {
        self.bids.last_key_value().map(|(p, _)| *p)
    }

    pub fn best_ask_price(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(p, _)| *p)
    }

    pub fn best_price(&self, side: Side) -> Option<u64> {
        match side {
            Side::Bid => self.best_bid_price(),
            Side::Ask => self.best_ask_price(),
        }
    }

    pub fn ask_prices(&self) -> impl Iterator<Item = u64> + '_ {
        self.asks.keys().copied()
    }

    pub fn bid_prices_desc(&self) -> impl Iterator<Item = u64> + '_ {
        self.bids.keys().rev().copied()
    }

    pub fn queue_mut(&mut self, side: Side, price: u64) -> Option<&mut VecDeque<Order>> {
        match side {
            Side::Bid => self.bids.get_mut(&price),
            Side::Ask => self.asks.get_mut(&price),
        }
    }

    pub fn remove_level(&mut self, side: Side, price: u64) {
        match side {
            Side::Bid => {
                self.bids.remove(&price);
            }
            Side::Ask => {
                self.asks.remove(&price);
            }
        }
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
