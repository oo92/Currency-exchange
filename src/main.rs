use tungstenite::{connect, Message};
use url::Url;
use chrono::Utc;
use ordered_float::OrderedFloat;
use serde_json::json;
use std::thread;
use std::time::Instant;
use models::{OrderType, OrderBook, LimitPrice, Data, Msg, Trade, Order};
mod models;

static BINANCE_WS_API: &str = "wss://stream.binance.com:9443/ws/btcshib@depth20@100ms";


fn print_order_book(order_book: &OrderBook) {
    // Clear screen
    clearscreen::clear().expect("Error clearing screen");

    // Print info
    println!("Status  : Connected");
    println!("Exchange: Bitstamp.net");
    println!("Symbol  : BTC/USD");
    println!("Time    : {} UTC", Utc::now().format("%a %b %e %T %Y"));
    println!();
    println!("         Bids                    Asks");

    // Print order book
    let mut i = 0;
    let mut rev_bids = order_book.bids.clone();
    rev_bids.reverse();
    for bid in rev_bids {
        if order_book.asks.len() > i {println!("{:08.8} @ {:08.2}\t{:08.8} @ {:08.2}",bid.size, bid.price, order_book.asks[i].size, order_book.asks[i].price);} 
        else {println!("{:08.8} @ {:08.2}",bid.size, bid.price);}
        i += 1;
        if i > 10 {break;}
    }
}

fn pull_binance() {
    let bnnc_url = format!("{}/ws/ethbtc@depth10@100ms", BINANCE_WS_API);
    let (mut socket, response) = connect(Url::parse(&bnnc_url).unwrap()).expect("Can't connect.");

    println!("Connected to binance stream.");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");
    for (ref header, ref header_value) in response.headers() {println!("- {}: {:?}", header, header_value);}

    loop {
        let msg = socket.read_message().expect("Error reading message");

        match msg {
            tungstenite::Message::Text(s) => {
                // Handle text message
                let parser: models::DepthStreamData = serde_json::from_str(&s).expect("Can't parse");
                for i in 0..parser.asks.len() {
                    println!("{}. ask: {}, size: {}",i, parser.asks[i].price, parser.asks[i].size);
                    println!("{}. bid: {}, size: {}",i, parser.bids[i].price, parser.bids[i].size);
                }
            },
            _ => {} // Ignore all other message types (binary, ping, and pong)
        };
    }
}


fn pull_bitstamp(){
    // Create Order Book data structure
    let mut order_book = OrderBook {bids: Vec::new(),asks: Vec::new(),};

    // Start timer
    let mut start = Instant::now();

    // Connect to Bitstamp.net
    let (mut socket, _response) =connect(Url::parse("wss://ws.bitstamp.net").unwrap()).expect("Can't connect");

    // Subscribe to Live Trades channel for BTC/USD
    socket.write_message(Message::Text(json!({"event": "bts:subscribe","data": {"channel": "live_trades_btcusd"}}).to_string(),).into(),).expect("Error sending message");

    // Subscribe to Live Orders channel for BTC/USD
    socket.write_message(Message::Text(json!({"event": "bts:subscribe","data": {"channel": "live_orders_btcusd"}}).to_string(),).into(),).expect("Error sending message");

    // Spin loop
    loop {
        // Read message from socket
        let msg = socket.read_message().expect("Error reading message");

        // Deserialize message
        let result: Result<Msg, serde_json::Error> = serde_json::from_str(msg.to_text().unwrap());

        let _value = match result {
            Ok(msg) => {
                // Match on message type
                if msg.event == "bts:subscription_succeeded" { } 
                else if msg.event == "trade" {} 
                else if msg.event == "order_created" {
                    if let Data::Order(order) = msg.data {
                        let limit_price = LimitPrice { price: OrderedFloat(order.price), size: OrderedFloat(order.amount), orders: vec![order.clone()],};
                        match (&order).order_type {
                            buy if buy == OrderType::Buy as u8 => {
                                let _value = match order_book.bids.binary_search(&limit_price) {
                                    Ok(i) => {
                                        order_book.bids[i].size += order.amount;
                                        order_book.bids[i].orders.push(order);
                                    }
                                    Err(i) => {
                                        // TODO: implement proper order sweeping
                                        if order_book.asks.len() > 0&& &limit_price.price >= &order_book.asks[0].price{for ask in order_book.asks.clone() {if limit_price.price >= ask.price {order_book.asks.remove(0);}}} 
                                        else {order_book.bids.insert(i, limit_price.clone());}
                                    }
                                };
                            }
                            ask if ask == OrderType::Sell as u8 => {
                                let _value = match order_book.asks.binary_search(&limit_price) {
                                    Ok(i) => {
                                        order_book.asks[i].size += order.amount;
                                        order_book.asks[i].orders.push(order);
                                    }
                                    Err(i) => {
                                        let mut rev_bids: Vec<LimitPrice> = order_book.bids.clone();
                                        rev_bids.reverse();
                                        // TODO: implement proper order sweeping
                                        if order_book.bids.len() > 0 && &limit_price.price <= &rev_bids[0].price{for bid in rev_bids {if limit_price.price <= bid.price {order_book.bids.remove(order_book.bids.len() - 1);}}} 
                                        else {order_book.asks.insert(i, limit_price.clone());}
                                    }
                                };
                            }
                            _ => (),
                        }
                    }
                } else if msg.event == "order_deleted" {
                    if let Data::Order(order) = msg.data {
                        let limit_price = LimitPrice {price: OrderedFloat(order.price),size: OrderedFloat(order.amount),orders: vec![order.clone()],};
                        match (&order).order_type {
                            buy if buy == OrderType::Buy as u8 => {
                                let _value = match order_book.bids.binary_search(&limit_price) {
                                    Ok(i) => {
                                        let _value =match order_book.bids[i].orders.binary_search(&order) {
                                                Ok(j) => {
                                                    order_book.bids[i].orders.remove(j);
                                                    if order_book.bids[i].orders.len() == 0 {order_book.bids.remove(i);} 
                                                    else {order_book.bids[i].size -= order.amount;}
                                                }
                                                Err(_) => (),
                                            };
                                    }
                                    Err(_) => (),
                                };
                            }
                            ask if ask == OrderType::Sell as u8 => {
                                let _value = match order_book.asks.binary_search(&limit_price) {
                                    Ok(i) => {
                                        let _value = match order_book.asks[i].orders.binary_search(&order) {
                                                Ok(j) => {
                                                    order_book.asks[i].orders.remove(j);
                                                    if order_book.asks[i].orders.len() == 0 {order_book.asks.remove(i);} 
                                                    else {order_book.asks[i].size -= order.amount;}
                                                }
                                                Err(_) => (),
                                            };
                                    }
                                    Err(_) => (),
                                };
                            }
                            _ => (),
                        }
                    }
                } else if msg.event == "order_changed" {
                    if let Data::Order(order) = msg.data {
                        let limit_price = LimitPrice {price: OrderedFloat(order.price),size: OrderedFloat(order.amount),orders: vec![order.clone()],};
                        match (&order).order_type {
                            buy if buy == OrderType::Buy as u8 => {
                                let _value = match order_book.bids.binary_search(&limit_price) {
                                    Ok(i) => {
                                        let _value =match order_book.bids[i].orders.binary_search(&order) {
                                                Ok(j) => {
                                                    order_book.bids[i].orders.remove(j);
                                                    if order_book.bids[i].orders.len() == 0 {order_book.bids.remove(i);} 
                                                    else {order_book.bids[i].size -= order.amount;}
                                                }
                                                Err(_) => (),
                                            };
                                        order_book.bids[i].size += order.amount;
                                        order_book.bids[i].orders.push(order);
                                    }
                                    Err(i) => {
                                        order_book.bids.insert(i, limit_price.clone());
                                        // TODO: implement proper order sweeping
                                        if order_book.asks.len() > 0 {if &limit_price.price >= &order_book.asks[0].price {for ask in order_book.asks.clone() {if limit_price.price >= ask.price {order_book.asks.remove(0);}} }}
                                    }
                                };
                            }
                            ask if ask == OrderType::Sell as u8 => {
                                let _value = match order_book.asks.binary_search(&limit_price) {
                                    Ok(i) => {
                                        let _value =match order_book.asks[i].orders.binary_search(&order) {
                                                Ok(j) => {
                                                    order_book.asks[i].orders.remove(j);
                                                    if order_book.asks[i].orders.len() == 0 { order_book.asks.remove(i); } 
                                                    else {order_book.asks[i].size -= order.amount;}
                                                }
                                                Err(_) => (),
                                            };
                                        order_book.asks[i].size += order.amount;
                                        order_book.asks[i].orders.push(order);
                                    }
                                    Err(i) => {
                                        order_book.asks.insert(i, limit_price.clone());
                                        // TODO: implement proper order sweeping
                                        if order_book.bids.len() > 0 {
                                            let mut rev_bids: Vec<LimitPrice> = order_book.bids.clone();
                                            rev_bids.reverse();
                                            if &limit_price.price <= &rev_bids[0].price {for bid in rev_bids {if limit_price.price <= bid.price {order_book.bids.remove(order_book.bids.len() - 1);}}}
                                        }
                                    }
                                };
                            }
                            _ => (),
                        }
                    }
                }
            }
            Err(_) => (),
        };
        if start.elapsed().as_millis() > 500 {
            start = Instant::now();
            print_order_book(&order_book);
        }
    }
}

fn main() {
    let binance_handle = thread::spawn(|| {
        pull_binance();
    });

    let bitstamp_handle = thread::spawn(|| {
        pull_bitstamp();
    });

    // You can also join the threads if you want main to wait for them to finish
    binance_handle.join().expect("The thread panicking");
    bitstamp_handle.join().expect("The thread panicking");
}