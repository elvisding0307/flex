#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub user_id: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub price: u64,
    pub quantity: u64,
    pub remaining: u64,
    pub timestamp: u64,
}

impl Order {
    pub fn new(
        id: u64,
        user_id: u64,
        side: Side,
        order_type: OrderType,
        price: u64,
        quantity: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            user_id,
            side,
            order_type,
            price,
            quantity,
            remaining: quantity,
            timestamp,
        }
    }

}
