use std::io;

#[cfg(target_family = "unix")]
// Привязка системной библиотеки libc
#[link_name = "c"]
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> i32;
}

fn syscall(message: String) -> io::Result<()> {
    let msg_ptr = message.as_ptr();
    let msg_len = message.len();

    let result = unsafe { write(1, msg_ptr, msg_len) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn main() {
    syscall("Hello from normal syscall!".to_string()).unwrap();
}
