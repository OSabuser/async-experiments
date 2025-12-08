use reqwest::Error;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://jsonplaceholder.typicode.com/posts/1";
    let start = Instant::now();

    // Запуск нескольких конкурентных задач одновременно
    let (response_1, response_2, response_3) =
        tokio::join!(reqwest::get(url), reqwest::get(url), reqwest::get(url));

    let elapsed = start.elapsed(); // 224 ms
    println!("Elapsed: {:.2?}", elapsed);
    println!("Response 1: {}", response_1.unwrap().text().await?);
    println!("Response 2: {}", response_2.unwrap().text().await?);
    println!("Respons 3: {}", response_3.unwrap().text().await?);
    Ok(())
}
