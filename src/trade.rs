#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trade {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: u64,
    pub quantity: u64,
}
