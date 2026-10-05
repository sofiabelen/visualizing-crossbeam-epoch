# Playground For Visualizing Epoch-Based Memory Reclamation and Lock-Free Concurrency

This is the home for the experiments used for demonstration purposes in my blog. In the series, so far:

- [Visualizing The ABA Problem: What crossbeam-epoch Solves](https://sofiabelen.github.io/projects/visualizing-the-aba-problem/)
    - Visualizing lock-free concurrency in Rust: reproducing the aba problem to begin understanding crossbeam. I promise lots of diagrams!
- [Using Loom to (Try To) Catch an ABA Bug in Lock-Free Rust](https://sofiabelen.github.io/projects/using-loom-to-catch-an-aba-bug/)
    - I'm exploring Loom for the first time! My goal test our broken lock-free stack with loom, to see if it can detect the ABA bug. It's my first time working with this tool, so I'm excited!
- [Visualizing crossbeam Epoch Based Reclamation](https://sofiabelen.github.io/projects/visualizing-crossbeam-epoch-based-reclamation/)
    - Continue the lock-free adventure by exploring the internals of crossbeam-epoch to answer two fundamental questions:
        1. How do we know when it's safe to drop?
        2. How do we atomically update both a pointer and a state flag in a single CAS instruction? This second question stems from the two-step process needed for deleting a node in a lock-free linked list.


## Directory Structure

```
src/
├── lib.rs                     --> configure loom shim
├── aba_problem.rs             --> reproduce the aba bug with the use of thread::sleep
├─── naive_lock_free_stack.rs  --> here's the loom test, the stack implementation is untouched
└── bin/
    ├── crossbeam_epochs.rs    --> inspect how retired memory is safely dropped (answers question 1)
    └── crossbeam_shared.rs    --> inspect Shared<T> tag bits to combine pointers and state flags for atomic CAS (answers question 2)
```

## Building

### Prerequisites

- Loom
- Miri (works with rust nightly).

### Running Tests

```
cargo test
```

#### Loom

Check out the [docs](https://docs.rs/loom/latest/loom/) for info on what loom is and how it works.

```
RUSTFLAGS="--cfg loom" RUST_BACKTRACE=1 cargo test --release aba_problem
```

#### Miri

```
cargo +nightly miri test aba_problem::tests::aba  
```

### Running Experiments Under `src/bin`

```
# Inspect safe deferred memory dropping in crossbeam-epoch
cargo run --bin crossbeam_epochs

# Inspect Shared<T> tag bits and atomic CAS operations
cargo run --bin crossbeam_shared
```