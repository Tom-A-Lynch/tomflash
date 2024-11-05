# tomflash-rs

hehe hehe

hehe hehe cubed

A Rust implementation of an autonomous AI agent that interacts on social media, manages cryptocurrency transactions, and maintains both short and long-term memory.

## Features

- Autonomous social media interaction
- ETH wallet management and transactions
- Short-term and long-term memory systems
- Vectorized memory storage and retrieval
- Automated posting and interaction scheduling

## Requirements

- Rust 1.75+
- PostgreSQL 15+
- OpenAI API key
- Twitter API credentials
- Ethereum node access (Mainnet)

## Configuration

Copy `.env.example` to `.env` and configure:


```
env
DATABASE_URL=postgres://user:pass@localhost/nousflash
OPENAI_API_KEY=your_key
TWITTER_API_KEY=your_key
TWITTER_API_SECRET=your_secret
TWITTER_ACCESS_TOKEN=your_token
TWITTER_ACCESS_SECRET=your_secret
ETH_MAINNET_RPC=your_rpc_url
```

## Running

```
bash
Build and run
cargo run --release
Run with development settings
cargo run
Run tests
cargo test
```

## Database

Uses Diesel ORM with PostgreSQL. Initialize the database:

```
bash
cargo install diesel_cli --no-default-features --features postgres
diesel setup
diesel migration run
```

## Architecture

The system consists of several key components:

1. Memory Systems
   - Short-term memory for recent interactions
   - Long-term memory with vector embeddings
   - Significance scoring for memory retention

2. Social Interaction
   - Autonomous posting
   - Reply generation
   - Timeline analysis

3. Wallet Management
   - ETH transaction handling
   - Balance monitoring
   - Transaction decision making

4. Pipeline System
   - Scheduled activation periods
   - Interaction frequency control
   - State management