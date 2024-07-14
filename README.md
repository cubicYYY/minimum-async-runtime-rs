# "Minimum" Async Runtime

Take it easy! Asynchronous programming in Rust is actually easy to understand once you get a sense of its philosophy. This repository is used for educational purposes and demonstrates how to build a minimal async runtime from scratch for Rust.

## Usage

```bash
git clone https://github.com/cubicYYY/minimum-async-runtime-rs.git
cd minimum-async-runtime-rs
```

Then:

```bash
RUST_LOG=INFO cargo run
# For Windows: 
# cmd /V /C "set RUST_LOG=INFO && cargo run"
```

## Overview

The executor is designed to schedule and run tasks asynchronously in a single-threaded environment, but you can modify it for a multi-threaded model easily.
It uses a task queue and a signaling mechanism to manage task execution and ensure that tasks are processed in order.

## Key Components

- **SignalReactor**: A simple signaling mechanism using an atomic boolean for task notification. In a real-world async runtime framework, a reactor is responsible for interacting with the kernel and "waking" the corresponding task. Waking a task involves putting the task somewhere the executor will find it and execute it.
- **Task**: A wrapper around a `Future` with no return value, allowing it to be scheduled and executed by the executor. It can be tricky to `spawn` a task with a return value. For more details, refer to [tokio](https://tokio.rs/) and [async-std](https://async.rs/) to check their implementation (`JoinHandle` and type erasure).
- **Executor Queue**: Consists of tasks that are ready to be polled, applying a simple FIFO scheduling policy to tasks in this simple runtime. In a real-world async runtime framework, additional technical details are introduced to prevent task starvation, but the basic idea is the same.

## Code Structure

- `SignalReactor`: Provides methods to notify and wait for signals.
- `Task`: Contains the future to be executed and the signal reactor. Implements the `Wake` trait for scheduling.
- `TimerFuture`: An example to show how `sleep` function works in an async runtime.
- `spawn`: Spawns a new task and adds it to the executor queue.
- `block_on`: Blocks the current thread until the given future completes, polling tasks in the executor queue.
