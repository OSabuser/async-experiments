use reqwest::Error;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://jsonplaceholder.typicode.com/posts/1";
    let start = Instant::now();
    let response_1 = reqwest::get(url).await?;
    let response_2 = reqwest::get(url).await?; // Выполняется только после завершения предыдущего
    let response_3 = reqwest::get(url).await?; // Выполняется только после завершения предыдущего
    let elapsed = start.elapsed(); // 550 ms
    println!("Elapsed: {:.2?}", elapsed);
    println!("Response 1: {}", response_1.text().await?);
    println!("Response 2: {}", response_2.text().await?); // Выполняется только после завершения предыдущего
    println!("Respons 3: {}", response_3.text().await?); // Выполняется только после завершения предыдущего
    Ok(())
}
