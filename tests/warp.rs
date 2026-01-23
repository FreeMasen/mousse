use mousse::Parser;

mod shared;
use shared::*;

#[tokio::test]
async fn parse_warp_stream() {
    const CT: u8 = 255;
    let filter = sse_filter();
    let res = warp::test::request()
        .path(&format!("/sse/{}", CT))
        .reply(&filter)
        .await;
    let body = str::from_utf8(res.body()).unwrap();
    let mut p = Parser::new(body);
    for _ in 0..=CT {
        p.next_event().unwrap();
    }
    assert!(p.next_event().is_none())
}
