/// Main abstraction - layer over epoll
/// ```
/// let queue = Poll::new().unwrap();
/// let id = 1;
/// register interest in events on a TcpStream
/// queue.registry().register(&stream, id, ...).unwrap();
/// let mut events = Vec::with_capacity(1);
//  This will block the current thread
/// queue.poll(&mut events, None).unwrap();
/// data is ready on one of the tracked streams
/// ```
use crate::ffi;

use std::{
    io::{self, Result},
    net::TcpStream,
    os::fd::AsRawFd,
};

type Events = Vec<ffi::Event>;

/// Event queue
pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Self> {
        let res = unsafe { ffi::epoll_create(1) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            registry: Registry { raw_fd: res },
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    // Only one thread can wait for event at the same time (blocking call + handle the notifications)!
    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        let fd = self.registry.raw_fd;

        // -1 if we want to block until an event occurs even though that might never happen
        let timeout = timeout.unwrap_or(-1);
        let max_events = events.capacity() as i32;
        let res = unsafe { ffi::epoll_wait(fd, events.as_mut_ptr(), max_events, timeout) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        unsafe { events.set_len(res as usize) }

        Ok(())
    }
}

pub struct Registry {
    raw_fd: i32,
}

/// Handle which allows to register interest in new events (for TcpStream)
impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        let mut event = ffi::Event {
            events: interests as u32,
            epoll_data: token,
        };

        let op = ffi::EPOLL_CTL_ADD;

        let res = unsafe { ffi::epoll_ctl(self.raw_fd, op, source.as_raw_fd(), &mut event) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.raw_fd) };
        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("ERROR: {err:?}");
        }
    }
}
