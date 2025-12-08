use std::arch::asm;

// Не инлайнить в процессе компиляции (при оптимизации)
#[inline(never)]
fn syscall(message: String) {
    let msg_ptr = message.as_ptr();
    let msg_len = message.len();

    unsafe {
        asm!("mov x16, 4", // Perform "write" operation
            "mov x0, 1", // Write to stdout
            "svc 0", // Software interrupt -> передача управления ядру ОС
            in("x1") msg_ptr, // Address of the message
            in("x2") msg_len, // Length of the message
            out("x16") _, // Ignore return value
            out("x0") _,
            lateout("x1") _,
            lateout("x2") _);
    }
}

fn main() {
    syscall("Hello from raw syscall!".to_string());
}
