use log::info;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

mod sleep;
pub use sleep::TimerFuture;

static SIGNAL: Lazy<Arc<SignalReactor>> = Lazy::new(|| Arc::new(SignalReactor::new()));
static EXECUTOR_QUEUE: Lazy<Mutex<VecDeque<Arc<Task>>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(1024)));

struct SignalReactor {
    signaled: std::sync::atomic::AtomicBool,
}

impl SignalReactor {
    fn new() -> Self {
        SignalReactor {
            signaled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn notify(&self) {
        self.signaled
            .store(true, std::sync::atomic::Ordering::Release);
    }

    fn wait(&self) {
        while !self.signaled.load(std::sync::atomic::Ordering::Acquire) {
            std::hint::spin_loop();
        }
        self.signaled
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    signal: Arc<SignalReactor>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let mut runnable = EXECUTOR_QUEUE.lock().unwrap();
        runnable.push_back(self.clone());
        info!("[Wake] Task appended to the EXECUTOR_QUEUE.");
        self.signal.notify();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let mut runnable = EXECUTOR_QUEUE.lock().unwrap();
        runnable.push_back(self.clone());
        info!("[Wake] Task appended to the EXECUTOR_QUEUE.");
        self.signal.notify();
    }
}

impl Task {
    fn new<F>(future: F) -> Arc<Self>
    where
        F: Future<Output = ()> + 'static + Send,
    {
        Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            signal: SIGNAL.clone(),
        })
    }
    fn waker_from(task: Arc<Self>) -> Waker {
        Waker::from(task)
    }
    fn poll(self: &Arc<Self>) -> Poll<()> {
        let waker = Self::waker_from(self.clone());
        let mut context = Context::from_waker(&waker);
        let mut future = self.future.lock().unwrap();
        future.as_mut().poll(&mut context)
    }
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static + Send,
{
    info!("[SPAWN] ...");
    let task = Task::new(future);
    EXECUTOR_QUEUE.lock().unwrap().push_back(task);
    SIGNAL.notify();
}

pub fn block_on<F, O>(future: F) -> O
where
    F: Future<Output = O>,
{
    let signal = SIGNAL.clone();
    pin_utils::pin_mut!(future);

    // We cannot wrap it in a Task struct: no need to set a waker to push the main task into the queue
    let noop_waker = waker_fn::waker_fn(|| {});
    let noop_cx = &mut Context::from_waker(&noop_waker);

    loop {
        info!("==== Executor loop ====");

        // Poll the main task to start
        info!("[*MAIN] Polling...");
        let main_status = future.as_mut().poll(noop_cx);
        info!("[Main task status] ready= {:?}", main_status.is_ready());
        if let Poll::Ready(res) = main_status {
            return res;
        }

        // Poll other tasks
        info!(
            "[INFO] Ready tasks in queue: {}",
            EXECUTOR_QUEUE.lock().unwrap().len()
        );
        while let Some(task) = EXECUTOR_QUEUE.lock().unwrap().pop_front() {
            info!("[sub] Polling...");
            let sub_status = task.poll();
            info!("[Subtask status] ready= {:?}", sub_status.is_ready());
            signal.notify();
        }

        info!("[Waiting...]");
        signal.wait();
    }
}
