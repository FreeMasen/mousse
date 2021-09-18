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
