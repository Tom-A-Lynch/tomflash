# Tomflash Engine Components

This directory contains the core processing engines for the Tomflash agent. Each engine is responsible for a specific aspect of the agent's cognitive and interaction capabilities.

## Core Components

### Memory Engines
- `short_term_mem.rs`: Processes recent interactions and context for immediate decision making
- `long_term_mem.rs`: Handles persistent memory storage and retrieval using vector embeddings
- `significance_scorer.rs`: Evaluates memory significance for storage decisions

### Interaction Engines
- `post_maker.rs`: Generates posts based on memory and context
- `post_retriever.rs`: Fetches and processes external content and interactions
- `post_sender.rs`: Handles the actual posting of content to platforms

### Financial Engines
- `wallet.rs`: Manages ETH wallet interactions and transaction decisions

### Support Components
- `prompts.rs`: Central storage for LLM prompt templates
- `ai.rs`: Core LLM interaction layer

## Architecture Notes

1. All engines follow the Result/Error pattern for consistent error handling
2. Memory operations are vectorized for efficient similarity search
3. Each engine implements relevant traits from `utils::traits`:
   - `Embeddable` for vector operations
   - `LLMFormattable` for AI interactions
   - `Scorable` for significance evaluation

## Data Flow

```
mermaid
graph TD
A[External Context] --> B[Short Term Memory]
B --> C[Post Maker]
D[Long Term Memory] --> C
C --> E[Post Sender]
C --> F[Significance Scorer]
F --> D
```

## Implementation Guidelines

1. Always use the central Config object for settings
2. Implement proper tracing at INFO and DEBUG levels
3. All external API calls should be retryable
4. Memory operations should be batched where possible
5. Follow the original Python patterns while leveraging Rust's type system

## Testing

Each engine should have:
- Unit tests for core logic
- Integration tests for external interactions
- Proper mocking of dependencies
- Benchmark tests for memory operations

## Original Python Reference
Key files to reference when implementing:
- `post_maker.py`: Lines 1-150 for core generation logic
- `short_term_mem.py`: Lines 1-100 for memory processing
- `long_term_mem.py`: Lines 1-200 for embedding operations
