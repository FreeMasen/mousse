# Mousse

An SSE encoder/decoder

## Usage

### Decode

```rust
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
    let mut stream = reqwest::get(&url)
        .await?
        .bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let string = String::from_utf8_lossy(&chunk);
        let mut parser = Parser::new(&string);
        println!("{:?}", parser.next_event());
    }
    Ok(())
}

```

#### Output

If you connect this to an sse server, you might see something like this.

```sh
cargo run --example reqwest -- http://localhost:8080/sse
Some(ServerSentEvent { comment: None, event: None, id: None, data: Some("next tick in 5"), retry: None })
Some(ServerSentEvent { comment: Some(""), event: None, id: None, data: None, retry: None })
Some(ServerSentEvent { comment: None, event: None, id: None, data: Some("next tick in 4"), retry: None })
Some(ServerSentEvent { comment: Some(""), event: None, id: None, data: None, retry: None })
Some(ServerSentEvent { comment: None, event: None, id: None, data: Some("next tick in 2"), retry: None })
Some(ServerSentEvent { comment: None, event: None, id: None, data: Some("next tick in 1"), retry: None })
```

### Encode

```rust
use mousse::ServerSentEvent;

fn main() {
    for i in 0..10 {
        println!(
            "{}",
            ServerSentEvent::builder()
                .data(&format!("{}: Hi I am a data field", i))
                .id(&i.to_string())
                .build()
        )
    }
}

```

#### Output

```sh
cargo run --example encode
id:0
data:0: Hi I am a data field


id:1
data:1: Hi I am a data field


id:2
data:2: Hi I am a data field


id:3
data:3: Hi I am a data field


id:4
data:4: Hi I am a data field


id:5
data:5: Hi I am a data field


id:6
data:6: Hi I am a data field


id:7
data:7: Hi I am a data field


id:8
data:8: Hi I am a data field


id:9
data:9: Hi I am a data field


```