use minimum_async_rt::{block_on, spawn, TimerFuture};

async fn print_msg(name: &str, tx: async_channel::Sender<u32>) {
    println!("!!!!!! Message: {name}!");
    let _ = tx.send(114514).await;
}

async fn what_universe_is() -> u32 {
    42
}

async fn long_io(tx: async_channel::Sender<u32>) {
    println!("Async I/O starts! Finish in 3 seconds...");
    TimerFuture::new(std::time::Duration::from_secs(3)).await;
    println!("Async I/O finished!");
    let _ = tx.send(1919810).await;
}

async fn demo() -> u32 {
    let (s, r) = async_channel::unbounded();
    println!("demo start");
    
    // Simulate a long I/O
    spawn(long_io(s.clone()));

    let universe = what_universe_is().await;
    println!("The universe equals to: {universe}");

    // Call an async function
    spawn(print_msg("Alice", s.clone()));

    // The same but wrapped
    let sender = s.clone();
    spawn(async move {
        print_msg("Bob", sender).await;
    });

    let recv_num = r.recv().await.unwrap();
    println!("Received: {recv_num}");
    let recv_num = r.recv().await.unwrap();
    println!("Received: {recv_num}");

    let recv_num = r.recv().await.unwrap();
    println!("Received: {recv_num}");

    println!("demo end");
    0
}

fn main() {
    env_logger::init();

    let res: u32 = block_on(demo());
    print!("DONE! RET= {res}");
}
