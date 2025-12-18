use std::{
    thread,
    time::{Duration, Instant},
};
mod future;
mod http;
use crate::http::Http;
use future::{Future, PollState};

fn main() {
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::Ready(()) => break,
            PollState::NotReady => {
                println!("Schedule next poll");
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

struct Coroutine {
    state: State,
}

/// Non-leaf future
impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();
    /// 1. The first thing we do is set the Output type to (). Since we won’t be returning anything, it
    ///     just makes our example simpler.
    /// 2. Next up is the implementation of the poll method. The first thing you notice is that we
    ///     write a loop instance that matches self.state. We do this so we can drive the state
    ///     machine forward until we reach a point where we can’t progress any further without getting
    ///     `PollState::NotReady` from one of our child futures.
    /// 3. If the state is State::Start, we know that this is the first time it was polled, so we run
    ///     whatever instructions we need until we reach the point where we get a new future that we
    ///     need to resolve.
    /// 4. When we call Http::get, we receive a future in return that we need to poll to completion
    ///     before we progress any further.
    ///  5. At this point, we change the state to State::Wait1 and we store the future we want to
    ///  resolve so we can access it in the next state.
    /// 6. Our state machine has now changed its state from `Start` to `Wait1`. Since we’re looping on
    ///     the match statement, we immediately progress to the next state and will reach the match arm
    ///     in `State::Wait1` on the next iteration.
    /// 7. The first thing we do in `Wait1` to call `poll` on the `Future` instance we’re waiting on.
    /// 8. If the future returns `PollState::NotReady`, we simply bubble that up to the caller by
    ///     breaking out of the loop and returning `NotReady`.
    /// 9. If the future returns `PollState::Ready` together with our data, we know that we can
    ///     execute the instructions that rely on the data from the first future and advance to the next state.
    ///     In our case, we only print out the returned data, so that’s only one line of code.
    /// 10. Next, we get to the point where we get a new future by calling `Http::get`. We set the state
    ///     to `Wait2`, just like we did when going from `State::Start` to `State::Wait1`.
    /// 11. Like we did the first time we got a future that we needed to resolve before we continue, we save
    ///     it so we can access it in `State::Wait2`.
    /// 12. Since we’re in a loop, the next thing that happens is that we reach the matching arm for Wait2,
    ///     and here, we repeat the same steps as we did for `State::Wait1` but on a different future.
    /// 13. If it returns `Ready` with our data, we act on it and we set the final state of our `Coroutine` to
    ///     `State::Resolved`. There is one more important change: this time, we want to communicate to
    ///     the caller that this future is done, so we break out of the loop and return `PollState::Ready`.
    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");
                    let fut = Box::new(Http::get("/600/HelloWorld1"));
                    self.state = State::Wait1(fut);
                }
                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/400/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt2) => {
                        println!("{txt2}");
                        self.state = State::Resolved;
                        break PollState::Ready(());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}
enum State {
    /// The coroutine has been created but hasn't been polled yet
    Start,
    /// First call to Http::get
    Wait1(Box<dyn Future<Output = String>>),
    /// Second call to Http::get
    Wait2(Box<dyn Future<Output = String>>),
    ///The future is resolved and there is no more work to do
    Resolved,
}
