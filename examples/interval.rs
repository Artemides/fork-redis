use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, Instant},
};

use futures::Stream;

struct Interval<F>
where
    F: Future<Output = ()> + 'static + Send,
{
    times: usize,
    future: F,
    interval: Duration,
}

impl<F> Stream for Interval<F>
where
    F: Future<Output = ()> + 'static + Send,
{
    type Item = ();

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.times == 0 {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(_) => {
                self.future = F::from(self.future);
                self.times -= 1;
                Poll::Ready(Some(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[tokio::main]
async fn main() {}

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
