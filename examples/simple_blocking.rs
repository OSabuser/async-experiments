use std::{error::Error, time::Instant};

async fn test_loop(task_number: u8) {
    for i in 0..10 {
        println!("Task #{task_number}: {}", i);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let _task_1 = test_loop(1).await;
    let _task_2 = test_loop(2).await;

    // Запуск нескольких конкурентных задач одновременно

    let elapsed = start.elapsed();
    println!("Все задачи выполнены за: {:.2?} мс", elapsed.as_millis());

    Ok(())
}
