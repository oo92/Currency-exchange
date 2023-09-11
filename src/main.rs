use tungstenite::{connect, Message};
use url::Url;
use chrono::Utc;
use ordered_float::OrderedFloat;
use serde_json::json;
use futures::SinkExt;
use models::{OrderType, OrderBook, LimitPrice, Data, Msg};
use tonic::{transport::Server, Request, Response, Status};
use std::{thread, fmt, time::Instant, sync::{Arc, Mutex}};
mod models;

static BINANCE_WS_API: &str = "wss://stream.binance.com:9443/ws/btcusd@depth20@100ms";

#[macro_use]
extern crate lazy_static;

lazy_static! {static ref COMBINED_ORDER_BOOK: Arc<Mutex<OrderBook>> = Arc::new(Mutex::new(OrderBook { bids: Vec::new(), asks: Vec::new() }));}

pub mod orderbook {tonic::include_proto!("orderbook");}

use crate::orderbook::{orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer}, Empty, Summary, Level};

pub struct OrderbookService;

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookService {
    type BookSummaryStream = futures::channel::mpsc::Receiver<Result<Summary, Status>>;

    async fn book_summary(&self, _request: Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (mut tx, rx) = futures::channel::mpsc::channel(4);

        tokio::spawn(async move {
            loop {
                let (spread, top_bids, top_asks) = {
                    let _combined_order_book = COMBINED_ORDER_BOOK.lock().unwrap();
        
                    let spread = if !_combined_order_book.bids.is_empty() && !_combined_order_book.asks.is_empty() {_combined_order_book.asks[0].price.into_inner() - _combined_order_book.bids[0].price.into_inner()} 
                    else {0.0};
        
                    let top_bids: Vec<Level> = _combined_order_book.bids.iter().take(10).map(|bid| Level { exchange: "Bitstamp".to_string(), price: bid.price.into_inner(), amount: bid.size.into_inner() }).collect();
                    let top_asks: Vec<Level> = _combined_order_book.asks.iter().take(10).map(|ask| Level { exchange: "Bitstamp".to_string(), price: ask.price.into_inner(), amount: ask.size.into_inner() }).collect();
        
                    (spread, top_bids, top_asks)
                };
        
                let summary = Summary {spread,bids: top_bids,asks: top_asks,};
        
                tx.send(Ok(summary)).await.expect("Channel send failed");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        

        Ok(Response::new(rx))
    }
}

impl From<url::ParseError> for AppError {fn from(err: url::ParseError) -> AppError {AppError::UrlParseError(err.to_string())}}

#[derive(Debug)]
enum AppError { ConnectionFailed(String), ParsingFailed(String), MessageError(String), AddrParseError(std::net::AddrParseError), UnknownError, UrlParseError(String)}

impl fmt::Display for AppError {fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {match self { AppError::ConnectionFailed(s) => write!(f, "Connection failed: {}", s), AppError::ParsingFailed(s) => write!(f, "Parsing failed: {}", s), AppError::MessageError(s) => write!(f, "Message error: {}", s), AppError::UnknownError => write!(f, "An unknown error occurred"), AppError::AddrParseError(e) => write!(f, "Address parsing error: {}", e), AppError::UrlParseError(e) => write!(f, "URL parsing error: {}", e)}}}

impl std::error::Error for AppError {}

fn print_order_book(order_book: &OrderBook) {
    clearscreen::clear().expect("Error clearing screen");

    println!("Status  : Connected");
    println!("Exchange: Bitstamp.net");
    println!("Symbol  : BTC/USD");
    println!("Time    : {} UTC", Utc::now().format("%a %b %e %T %Y"));
    println!();
    println!("         Bids                    Asks");

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

fn pull_binance() -> Result<(), AppError> {
    let bnnc_url = format!("{}/ws/ethbtc@depth10@100ms", BINANCE_WS_API);

    let (mut socket, response) = connect(Url::parse(&bnnc_url)?).map_err(|_| AppError::ConnectionFailed("Binance".to_string()))?;

    println!("Connected to binance stream.");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");
    for (ref header, ref header_value) in response.headers() {println!("- {}: {:?}", header, header_value);}

    loop {
        let msg = socket.read_message().map_err(|e| AppError::MessageError(e.to_string()))?;

        match msg {
            tungstenite::Message::Text(s) => {
                // Handle text message
                let parser: models::DepthStreamData = serde_json::from_str(&s).map_err(|_| AppError::ParsingFailed(s.clone()))?;
                for i in 0..parser.asks.len() {
                    println!("{}. ask: {}, size: {}",i, parser.asks[i].price, parser.asks[i].size);
                    println!("{}. bid: {}, size: {}",i, parser.bids[i].price, parser.bids[i].size);
                }
            },
            _ => {}
        };
    }
}


fn pull_bitstamp(){
    let mut order_book = OrderBook {bids: Vec::new(),asks: Vec::new(),};
    let mut start = Instant::now();
    let (mut socket, _response) =connect(Url::parse("wss://ws.bitstamp.net").unwrap()).expect("Can't connect");

    socket.write_message(Message::Text(json!({"event": "bts:subscribe","data": {"channel": "live_trades_btcusd"}}).to_string(),).into(),).expect("Error sending message");
    socket.write_message(Message::Text(json!({"event": "bts:subscribe","data": {"channel": "live_orders_btcusd"}}).to_string(),).into(),).expect("Error sending message");

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let result: Result<Msg, serde_json::Error> = serde_json::from_str(msg.to_text().unwrap());

        let _value = match result {
            Ok(msg) => {
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

#[tokio::main]
async fn main() {
    match run_app().await {
        Ok(_) => println!("Application exited gracefully."),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_app() -> Result<(), AppError> {
    let binance_handle = thread::spawn(|| pull_binance());
    let bitstamp_handle = thread::spawn(|| pull_bitstamp());

    let binance_result = binance_handle.join().map_err(|_| AppError::UnknownError)?;
    let bitstamp_result = bitstamp_handle.join().map_err(|_| AppError::UnknownError)?;

    let addr = "127.0.0.1:50051".parse().map_err(AppError::AddrParseError)?;
    let orderbook_service = OrderbookService {};

    println!("gRPC Server started on {}", addr);

    Server::builder().add_service(OrderbookAggregatorServer::new(orderbook_service)).serve(addr).await;

    Ok(())
}