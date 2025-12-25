use crate::http::Http;

use crate::future::{Future, PollState};
use crate::runtime::Waker;
fn main() {
    let mut executor = runtime::init();
    executor.block_on(async_main());
}

// =================================
// We rewrite this:
// =================================

// coroutine fn async_main() {
// println!("Program starting");
// let txt = Http::get("/600/HelloAsyncAwait").wait;
// println!("{txt}");
// let txt = Http::get("/400/HelloAsyncAwait").wait;
// println!("{txt}");

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output = String> {
    Coroutine0::new()
}

enum State0 {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

#[derive(Default)]
struct Stack0 {
    counter: Option<usize>,
}
struct Coroutine0 {
    stack: Stack0,
    state: State0,
}

impl Coroutine0 {
    fn new() -> Self {
        Self {
            state: State0::Start,
            stack: Stack0::default(),
        }
    }
}

impl Future for Coroutine0 {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
            match self.state {
                State0::Start => {
                    // ---- Code you actually wrote ----
                    println!("Program starting");
                    // initialize stack (hoist variables)
                    self.stack.counter = Some(0);
                    println!("counter was initialized: {}", self.stack.counter.unwrap());
                    // ---------------------------------
                    let fut1 = Box::new(Http::get("/600/HelloAsyncAwait"));
                    self.state = State0::Wait1(fut1);

                    //TODO: save stack
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll(waker) {
                        PollState::Ready(txt) => {
                            // Restore stack
                            let mut counter = self.stack.counter.take().unwrap();
                            println!("counter was extracted [1]: {}", counter);

                            println!("{txt}");
                            counter += 1;

                            // ---------------------------------
                            let fut2 = Box::new(Http::get("/400/HelloAsyncAwait"));

                            // save stack
                            self.stack.counter = Some(counter);
                            self.state = State0::Wait2(fut2);
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Wait2(ref mut f2) => {
                    match f2.poll(waker) {
                        PollState::Ready(txt) => {
                            // Restore stack
                            let mut counter = self.stack.counter.take().unwrap();
                            println!("counter was extracted [2]: {}", counter);
                            // ---- Code you actually wrote ----
                            println!("{txt}");

                            counter += 1;
                            println!("Received {} responses", counter);

                            // ---------------------------------
                            self.state = State0::Resolved;
                            break PollState::Ready(String::new());
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}
