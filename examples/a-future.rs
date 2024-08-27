use std::{
    future::Future,
    io,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, Instant},
};

use futures::{
    channel::oneshot,
    task::{waker, ArcWake},
};
use tokio::net::TcpListener;

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            scheduled: rx,
            sender: tx,
        }
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static + Send) {
        Task::spawn(future, self.sender.clone());
    }
}

struct Task {
    task_future: Mutex<TaskFuture>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl Task {
    fn poll(self: &Arc<Self>) {
        let waker = waker(self.clone());
        let mut cx = Context::from_waker(&waker);
        let mut task_future = self.task_future.try_lock().unwrap();
        task_future.poll(&mut cx);
    }

    fn spawn(future: impl Future<Output = ()> + 'static + Send, sender: mpsc::Sender<Arc<Task>>) {
        let task = Task {
            sender: sender.clone(),
            task_future: Mutex::new(TaskFuture::new(future)),
        };
        sender.send(Arc::new(task)).unwrap();
    }
    fn schedule(self: &Arc<Self>) {
        self.sender.send(self.clone()).unwrap();
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    state: Poll<()>,
}

impl TaskFuture {
    fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static + Send,
    {
        Self {
            future: Box::pin(future),
            state: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        if self.state == Poll::Pending {
            self.state = self.future.as_mut().poll(cx);
        }
    }
}

fn main() {
    let mut mini_tokio = MiniTokio::new();
    let delay_future = Delay {
        until: Instant::now() + Duration::from_secs(3),
        waker: None,
    };
    mini_tokio.spawn(async {
        let state = delay_future.await;
        println!("state: {state}");
    });

    mini_tokio.run();
}

struct Delay {
    until: Instant,
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.until < Instant::now() {
            return Poll::Ready("Ok");
        }

        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();
            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
            }
        } else {
            let until = self.until;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            thread::spawn(move || {
                let now = Instant::now();
                if until > now {
                    thread::sleep(until - now);
                }

                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
        }

        Poll::Pending
    }
}
