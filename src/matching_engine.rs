use std::cmp::min;

use crate::order::{Order, OrderType, Side};
use crate::order_book::OrderBook;
use crate::trade::Trade;

pub struct MatchingEngine {
    order_book: OrderBook,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            order_book: OrderBook::new(),
        }
    }

    pub fn execute(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut remaining = order.quantity;
        let opposing_side = match order.side {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        };

        let prices: Vec<u64> = match opposing_side {
            Side::Ask => self.order_book.ask_prices().collect(),
            Side::Bid => self.order_book.bid_prices_desc().collect(),
        };

        for price in prices {
            if remaining == 0 {
                break;
            }

            if order.order_type == OrderType::Limit {
                let price_ok = match order.side {
                    Side::Bid => price <= order.price,
                    Side::Ask => price >= order.price,
                };
                if !price_ok {
                    break;
                }
            }

            let queue = match self.order_book.queue_mut(opposing_side, price) {
                Some(q) => q,
                None => continue,
            };

            let mut self_trades: Vec<Order> = Vec::new();

            while let Some(mut maker) = queue.pop_front() {
                if remaining == 0 {
                    queue.push_front(maker);
                    break;
                }

                if maker.user_id == order.user_id {
                    self_trades.push(maker);
                    continue;
                }

                let trade_qty = min(remaining, maker.remaining);
                maker.remaining -= trade_qty;
                remaining -= trade_qty;

                trades.push(Trade {
                    maker_order_id: maker.id,
                    taker_order_id: order.id,
                    price,
                    quantity: trade_qty,
                });

                if maker.remaining > 0 {
                    queue.push_front(maker);
                }
            }

            // Return self-trade orders to the back, preserving relative order
            for st in self_trades {
                queue.push_back(st);
            }

            if queue.is_empty() {
                self.order_book.remove_level(opposing_side, price);
            }
        }

        // If limit order has leftover quantity, add it to the book
        if remaining > 0 && order.order_type == OrderType::Limit {
            let mut resting = order;
            resting.remaining = remaining;
            self.order_book.add(resting);
        }

        trades
    }

    pub fn order_book(&self) -> &OrderBook {
        &self.order_book
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::{Order, OrderType, Side};

    fn limit_buy(id: u64, user_id: u64, price: u64, qty: u64) -> Order {
        Order::new(id, user_id, Side::Bid, OrderType::Limit, price, qty, 0)
    }

    fn limit_sell(id: u64, user_id: u64, price: u64, qty: u64) -> Order {
        Order::new(id, user_id, Side::Ask, OrderType::Limit, price, qty, 0)
    }

    fn market_buy(id: u64, user_id: u64, qty: u64) -> Order {
        Order::new(id, user_id, Side::Bid, OrderType::Market, 0, qty, 0)
    }

    fn market_sell(id: u64, user_id: u64, qty: u64) -> Order {
        Order::new(id, user_id, Side::Ask, OrderType::Market, 0, qty, 0)
    }

    #[test]
    fn test_full_match_buy_takes_ask() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 100, 10));
        let trades = engine.execute(limit_buy(2, 1, 100, 10));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].maker_order_id, 1);
        assert_eq!(trades[0].taker_order_id, 2);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 10);
    }

    #[test]
    fn test_full_match_sell_takes_bid() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_buy(1, 2, 100, 10));
        let trades = engine.execute(limit_sell(2, 1, 100, 10));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].maker_order_id, 1);
        assert_eq!(trades[0].taker_order_id, 2);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 10);
    }

    #[test]
    fn test_partial_fill_resting_remains() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 100, 10));
        let trades = engine.execute(limit_buy(2, 1, 100, 5));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(engine.order_book().best_ask_price(), Some(100));
    }

    #[test]
    fn test_taker_partially_filled_remainder_rests() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 100, 5));
        let trades = engine.execute(limit_buy(2, 1, 100, 10));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(engine.order_book().best_bid_price(), Some(100));
        assert_eq!(engine.order_book().best_ask_price(), None);
    }

    #[test]
    fn test_price_time_priority() {
        let mut engine = MatchingEngine::new();
        // Two asks at the same price, placed at different times
        engine.execute(limit_sell(1, 2, 100, 5));
        engine.execute(limit_sell(2, 3, 100, 5));

        let trades = engine.execute(limit_buy(3, 1, 100, 7));

        // First 5 should come from order 1 (earlier timestamp), next 2 from order 2
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].maker_order_id, 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].maker_order_id, 2);
        assert_eq!(trades[1].quantity, 2);
    }

    #[test]
    fn test_multiple_price_levels() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 101, 5));
        engine.execute(limit_sell(2, 3, 100, 5));

        let trades = engine.execute(limit_buy(3, 1, 101, 10));

        // Should match 100 first (best ask), then 101
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 100); // Best price first
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].price, 101);
        assert_eq!(trades[1].quantity, 5);
    }

    #[test]
    fn test_limit_order_no_match_rests_in_book() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 110, 10));

        // Buy at 100, best ask is 110 — no match
        let trades = engine.execute(limit_buy(2, 1, 100, 10));

        assert!(trades.is_empty());
        assert_eq!(engine.order_book().best_bid_price(), Some(100));
        assert_eq!(engine.order_book().best_ask_price(), Some(110));
    }

    #[test]
    fn test_market_buy_fills_at_best_ask() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 100, 5));
        engine.execute(limit_sell(2, 3, 105, 5));

        let trades = engine.execute(market_buy(3, 1, 8));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].price, 105);
        assert_eq!(trades[1].quantity, 3);
    }

    #[test]
    fn test_market_buy_with_no_liquidity() {
        let mut engine = MatchingEngine::new();
        let trades = engine.execute(market_buy(1, 1, 10));

        assert!(trades.is_empty());
    }

    #[test]
    fn test_self_trade_prevention() {
        let mut engine = MatchingEngine::new();
        // User 1 places an ask
        engine.execute(limit_sell(1, 1, 100, 10));
        // User 1 tries to buy — should not match with their own order
        let trades = engine.execute(limit_buy(2, 1, 100, 10));

        assert!(trades.is_empty());
        // Both orders remain in the book
        assert_eq!(engine.order_book().best_ask_price(), Some(100));
        assert_eq!(engine.order_book().best_bid_price(), Some(100));
    }

    #[test]
    fn test_self_trade_mixed_with_other_orders() {
        let mut engine = MatchingEngine::new();
        // User 1 sells at 100
        engine.execute(limit_sell(1, 1, 100, 5));
        // User 2 sells at 100
        engine.execute(limit_sell(2, 2, 100, 5));

        // User 1 buys — should skip own order, match user 2's order
        let trades = engine.execute(limit_buy(3, 1, 100, 7));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].maker_order_id, 2);
        assert_eq!(trades[0].quantity, 5);
        // User 1's own ask remains
        assert_eq!(engine.order_book().best_ask_price(), Some(100));
    }

    #[test]
    fn test_market_order_does_not_rest() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 100, 5));

        let trades = engine.execute(market_buy(2, 1, 10));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        // Extra 5 from the market buy is lost (no resting)
        assert_eq!(engine.order_book().best_bid_price(), None);
    }

    #[test]
    fn test_zero_quantity_order() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_sell(1, 2, 100, 5));
        let trades = engine.execute(limit_buy(2, 1, 100, 0));

        assert!(trades.is_empty());
    }

    #[test]
    fn test_market_sell_fills_at_best_bid() {
        let mut engine = MatchingEngine::new();
        engine.execute(limit_buy(1, 2, 105, 5));
        engine.execute(limit_buy(2, 3, 100, 5));

        let trades = engine.execute(market_sell(3, 1, 8));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 105); // Best bid first
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].price, 100);
        assert_eq!(trades[1].quantity, 3);
    }

    #[test]
    fn test_match_at_maker_price() {
        let mut engine = MatchingEngine::new();
        // Maker asks 100, taker bids 105 — trade should happen at 100 (maker's price)
        engine.execute(limit_sell(1, 2, 100, 10));
        let trades = engine.execute(limit_buy(2, 1, 105, 10));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 100);
    }

    #[test]
    fn test_self_trade_does_not_block_worse_prices() {
        let mut engine = MatchingEngine::new();
        // User 1 has ask at 100 (best price)
        engine.execute(limit_sell(1, 1, 100, 5));
        // User 2 has ask at 101 (worse price)
        engine.execute(limit_sell(2, 2, 101, 5));

        // User 1 sends market buy — should skip own ask at 100 and match user 2 at 101
        let trades = engine.execute(market_buy(3, 1, 5));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].maker_order_id, 2);
        assert_eq!(trades[0].price, 101);
        assert_eq!(trades[0].quantity, 5);
        // User 1's own ask still rests
        assert_eq!(engine.order_book().best_ask_price(), Some(100));
    }
}
