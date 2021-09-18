use core::convert::Infallible;
use futures::{Stream, StreamExt};
use mousse::Parser;
use regex_generate::{Generator, DEFAULT_MAX_REPEAT};
use warp::{sse::Event, Filter, Rejection, Reply};

fn sse(ct: u8) -> impl Stream<Item = Result<Event, Infallible>> {
    futures::stream::iter((0..=ct).into_iter().map({
        |i| {
            let mut gen =
                Generator::new(r"[^\r\n]*", rand::thread_rng(), DEFAULT_MAX_REPEAT).unwrap();
            let mut data = Vec::new();
            gen.generate(&mut data).unwrap();
            Ok(Event::default()
                .id(i.to_string())
                .data(String::from_utf8_lossy(&data)))
        }
    }))
}

fn sse_filter() -> impl Filter<Extract = (impl Reply,), Error = Rejection> {
    warp::path!(u8).map(|ct: u8| warp::sse::reply(warp::sse::keep_alive().stream(sse(ct))))
}

#[tokio::test]
async fn parse_warp_stream() {
    const CT: u8 = 255;
    let filter = sse_filter();
    let res = warp::test::request()
        .path(&format!("/{}", CT))
        .filter(&filter)
        .await
        .unwrap();
    let mut res = res.into_response();
    let body = res.body_mut();
    let mut full_body = String::new();
    while let Some(part) = body.next().await {
        let part = part.unwrap();
        full_body.push_str(&String::from_utf8_lossy(&part));
    }
    let mut p = Parser::new(&full_body);
    for _ in 0..=CT {
        p.next_event().unwrap();
    }
    assert!(p.next_event().is_none())
}
