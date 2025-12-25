use crate::{
    Future,
    future::PollState,
    runtime::{self, Waker, reactor},
};
use mio::Interest;
use std::io::{ErrorKind, Read, Write};

fn get_request(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}

pub struct Http;
// http://127.0.0.1:8080/1000/HelloWorld
impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

// Leaf-future
struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    // Path of GET request
    path: String,
    id: usize,
}

impl HttpGetFuture {
    fn new(path: &str) -> HttpGetFuture {
        let id = reactor().next_id();
        HttpGetFuture {
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
            id,
        }
    }

    /// Sends the GET request and reads the response
    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream
            .write_all(get_request(&self.path).as_bytes())
            .unwrap();
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    /// # States
    /// (1) Not started (`self.stream` is `None`)
    /// (2) Pending (`self.stream` is `Some` and read to `stream.read` returns `WouldBlock`)
    /// (3) Resolved (`self.stream` is `Some` and read to `stream.read` returns 0 bytes)
    type Output = String;
    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        // Check if poll is launching for the first time
        if self.stream.is_none() {
            println!("First poll phase - START OPERATION");
            self.write_request();

            let stream = self.stream.as_mut().unwrap();
            runtime::reactor().register(stream, Interest::READABLE, self.id);
            runtime::reactor().set_waker(waker, self.id);
        }
        let mut buff = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    // All data has been read
                    let s = String::from_utf8_lossy(&self.buffer);
                    runtime::reactor().deregister(self.stream.as_mut().unwrap(), self.id);
                    break PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    // Some data has been read
                    self.buffer.extend(&buff[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    // Data isn't ready yet or there is more data but we haven't received it yet
                    runtime::reactor().set_waker(waker, self.id);
                    break PollState::NotReady;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    // Interrupted by a signal
                    continue;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
