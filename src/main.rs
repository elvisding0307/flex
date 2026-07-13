fn main() {
    println!("=== Flex Matching Engine Demo ===\n");

    let mut engine = flex::MatchingEngine::new();

    // Place resting orders to build the book
    println!("--- Placing resting orders ---");
    println!("User 2: SELL 10 @ 102");
    engine.execute(flex::Order::new(
        1, 2, flex::Side::Ask, flex::OrderType::Limit, 102, 10, 0,
    ));
    println!("User 2: SELL 5 @ 101");
    engine.execute(flex::Order::new(
        2, 2, flex::Side::Ask, flex::OrderType::Limit, 101, 5, 1,
    ));
    println!("User 3: SELL 8 @ 100");
    engine.execute(flex::Order::new(
        3, 3, flex::Side::Ask, flex::OrderType::Limit, 100, 8, 2,
    ));
    println!("User 4: BUY 6 @ 98");
    engine.execute(flex::Order::new(
        4, 4, flex::Side::Bid, flex::OrderType::Limit, 98, 6, 3,
    ));
    println!("User 5: BUY 4 @ 99");
    engine.execute(flex::Order::new(
        5, 5, flex::Side::Bid, flex::OrderType::Limit, 99, 4, 4,
    ));

    print_book(&engine);

    // User 1 sends a market buy — should walk up the ask side
    println!("--- User 1: MARKET BUY 15 ---");
    let trades = engine.execute(flex::Order::new(
        6, 1, flex::Side::Bid, flex::OrderType::Market, 0, 15, 5,
    ));

    for t in &trades {
        println!(
            "  TRADE: qty={} @ price={} | maker={} taker={}",
            t.quantity, t.price, t.maker_order_id, t.taker_order_id
        );
    }
    println!("  Total trades: {}\n", trades.len());

    print_book(&engine);

    // User 1 places a limit buy that doesn't cross
    println!("--- User 1: LIMIT BUY 5 @ 100 ---");
    let trades = engine.execute(flex::Order::new(
        7, 1, flex::Side::Bid, flex::OrderType::Limit, 100, 5, 6,
    ));

    for t in &trades {
        println!(
            "  TRADE: qty={} @ price={} | maker={} taker={}",
            t.quantity, t.price, t.maker_order_id, t.taker_order_id
        );
    }
    println!("  Total trades: {}\n", trades.len());

    print_book(&engine);

    // User 3 places a sell that crosses the bid
    println!("--- User 3: LIMIT SELL 3 @ 99 ---");
    let trades = engine.execute(flex::Order::new(
        8, 3, flex::Side::Ask, flex::OrderType::Limit, 99, 3, 7,
    ));

    for t in &trades {
        println!(
            "  TRADE: qty={} @ price={} | maker={} taker={}",
            t.quantity, t.price, t.maker_order_id, t.taker_order_id
        );
    }
    println!("  Total trades: {}\n", trades.len());

    print_book(&engine);
}

fn print_book(engine: &flex::MatchingEngine) {
    let book = engine.order_book();
    println!("  Order Book:");
    println!(
        "    Best bid: {}",
        book.best_bid_price()
            .map_or("none".to_string(), |p| p.to_string())
    );
    println!(
        "    Best ask: {}",
        book.best_ask_price()
            .map_or("none".to_string(), |p| p.to_string())
    );
    println!();
}
