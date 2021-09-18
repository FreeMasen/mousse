use futures::stream::StreamExt;
use mousse::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            eprintln!("expected uri argument\ncargo run -- <uri>");
            std::process::exit(1);
        }
    };
    let mut stream = reqwest::get(&url).await?.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let string = String::from_utf8_lossy(&chunk);
        let mut parser = Parser::new(&string);
        println!("{:?}", parser.next_event());
    }
    Ok(())
}
