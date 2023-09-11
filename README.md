# README: Rust Exchange Simulator

## Introduction:

This code provides an implementation of an order book aggregator, sourcing data from the Binance and Bitstamp exchanges. It offers both real-time updates and print functionalities for BTC/USD trade pairs.

## Features:
Connects to both Binance and Bitstamp using WebSocket API.
Provides real-time order book updates (bid & ask data).
Offers summary of the order book through tonic gRPC services.
Prints the order book in a formatted manner.
## Dependencies:
- `tungstenite`:For WebSocket connections.
- `url`: For parsing URLs.
- `chrono`: To get the current UTC time.
- `ordered_float`: To maintain floating numbers in order.
- `serde_json`: For JSON serialization and deserialization.
- `futures`: For future and async programming.
- `tonic`: gRPC framework for Rust.
- `std`: Standard library modules for thread, time, synchronization, etc.
- `clearscreen`: Clears the terminal screen for neat display.
- `models`: A module (presumably defined elsewhere) for data structures.

## Constants:
- `BINANCE_WS_API`: The WebSocket API URL endpoint for Binance.

## Core Components:
- `COMBINED_ORDER_BOOK`: A static, lazy-initialized, mutex-protected OrderBook that maintains a combined list of bids and asks.

- `OrderbookService` Struct: Implements the gRPC OrderbookAggregator service trait which serves the book_summary function. This function returns a summary of the order book.

- `AppError` Enum: An enumeration representing potential errors the app might encounter such as connection failures, parsing failures, etc.

- `print_order_book` Function: This function takes an OrderBook and prints its bids and asks in a readable format.

- `pull_binance` Function: Continuously fetches and processes order book data from Binance.

- `pull_bitstamp` Function: Continuously fetches and processes order book data from Bitstamp.

- `main` Function: The entry point of the application. It manages the app's lifecycle and error handling.

## How to Run:

```
cargo clean
cargo build
cargo run
```

## Error Handling:

Custom AppError enum is provided to handle different errors like connection failures, parsing errors, etc.
Implementations of the Display and Error traits are provided for the AppError for better error representation and propagation.

## Limitations:
The current implementation only supports the BTC/USD trading pair for binance and ETH/BTC for bitstamp.
Error handling may need improvements for more specific error causes.

## Recommendations for Future Iterations:
Support multiple trading pairs.
Enhance error handling with retries, especially for network-related issues.
Potentially expand to include more exchanges for a comprehensive order book.