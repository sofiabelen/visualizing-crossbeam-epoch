# Playground For Visualizing Epoch-Based Memory Reclamation and Lock-Free Concurrency

This is the home for the experiments used for demonstration purposes in my blog. In the series, so far:

- [Visualizing The ABA Problem: What crossbeam-epoch Solves](https://sofiabelen.github.io/projects/visualizing-the-aba-problem/)
- [Using Loom to (Try To) Catch an ABA Bug in Lock-Free Rust](https://sofiabelen.github.io/projects/using-loom-to-catch-an-aba-bug/)

# Building

## Prerequisites

- Loom
- Miri (works with rust nightly).

## Running Tests

```
cargo test
```

## Running Loom Tests

Check out the [docs](https://docs.rs/loom/latest/loom/) for info on what loom is and how it works.

```
RUSTFLAGS="--cfg loom" RUST_BACKTRACE=1 cargo test --release aba_problem
```
