use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use std::thread;
use futures::task::{self, ArcWake};
use crossbeam::channel;

fn main() {
    let mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        // first task
        spawn(async {
            delay(Duration::from_millis(100)).await;
            println!("world");
        });

        // second task
        spawn(async {
            println!("hello");
        });

        // force exit
        delay(Duration::from_millis(200)).await;
        std::process::exit(0);
    });

    mini_tokio.run();
}

struct MiniTokio {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = channel::unbounded();

        MiniTokio { scheduled, sender }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&self) {
        // Set the CURRENT thread-local to point to the current executer.
        CURRENT.with(|cell| {
            *cell.borrow_mut() = Some(self.sender.clone());
        });
        // executer loop
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

// tokio::spawn
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    CURRENT.with(|cell| {
        let borrow = cell.borrow();
        let sender = borrow.as_ref().unwrap();
        Task::spawn(future, sender);
    });
}

async fn delay(dur: Duration)
{
    struct Delay {
        when: Instant,
        waker: Option<Arc<Mutex<Waker>>>,
    }

    impl Future for Delay {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            // spawn the timer thread
            if let Some(waker) = &self.waker {
                let mut waker = waker.lock().unwrap();

                // check if the stored waker matches the current tasks waker
                if !waker.will_wake(cx.waker()) {
                    *waker = cx.waker().clone();
                }
            } else {
                let when = self.when;
                let waker = Arc::new(Mutex::new(cx.waker().clone()));
                self.waker = Some(waker.clone());

                thread::spawn(move || {
                    let now = Instant::now();

                    if now < when {
                        thread::sleep(when - now);
                    }

                    let waker = waker.lock().unwrap();
                    waker.wake_by_ref();
                });
            }

            if Instant::now() >= self.when {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
    let future = Delay {
        when: Instant::now() + dur,
        waker: None,
    };

    future.await;
}

thread_local! {
    static CURRENT: RefCell<Option<channel::Sender<Arc<Task>>>> =
        RefCell::new(None);
}

struct Task {
    // use `Mutex` to make task as `Sync`
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executer: channel::Sender<Arc<Task>>,
}

impl Task {
    fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.try_lock().unwrap();

        // polling future
        let _ = future.as_mut().poll(&mut cx);
    }
    // create new task from future and send it to channel
    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executer: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _ = arc_self.executer.send(arc_self.clone());
    }
}
