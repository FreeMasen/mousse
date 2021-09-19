use core::convert::Infallible;
use fantoccini::{Client, ClientBuilder, Locator};
use mousse::ServerSentEvent;
use warp::{http::Response, hyper::Body, Filter};

async fn standup_server(rx: tokio::sync::oneshot::Receiver<()>, port: u16) {
    let dir = warp::fs::dir("tests/browser_client_assets");
    let sse = warp::path!("sse").and(warp::get()).and_then(|| async {
        let end = std::iter::once(Ok(format!(
            "{}",
            ServerSentEvent::builder().event("close").build()
        )
        .as_bytes()
        .to_vec()));
        let stream = futures::stream::iter(
            (0..255)
                .map(|id| {
                    Result::<_, Infallible>::Ok(
                        format!(
                            "{}",
                            ServerSentEvent::builder()
                                .data("this is some data")
                                .id(&id.to_string())
                                .build()
                        )
                        .as_bytes()
                        .to_vec(),
                    )
                })
                .chain(end),
        );
        let body = Body::wrap_stream(stream);
        Result::<_, Infallible>::Ok(
            Response::builder()
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .body(body)
                .unwrap(),
        )
    });
    let (_addr, server) = warp::serve(sse.or(dir).with(warp::log("chrome-client-test-server")))
        .bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            rx.await.ok();
        });
    tokio::task::spawn(server);
}

async fn run_browser(c: &mut Client, port: u16) {
    let max_wait = std::time::Duration::from_secs(5);
    c.goto(&format!("http://localhost:{}", port)).await.unwrap();
    c.wait()
        .at_most(max_wait)
        .for_element(Locator::Css("#main"))
        .await
        .map_err(|e| {
            panic!("failed to wait for #main: {}", e);
        })
        .unwrap();
    c.wait()
        .at_most(max_wait)
        .for_element(Locator::Css("#list"))
        .await
        .map_err(|e| {
            panic!("failed to wait for #list: {}", e);
        })
        .unwrap();
    for i in 0..=255 {
        c.wait()
            .at_most(max_wait)
            .for_element(Locator::Css(&format!("#message-{}", i)))
            .await
            .map_err(|e| {
                panic!("failed to wait for #message-{}: {}", i, e);
            })
            .unwrap();
    }
    c.close().await.unwrap();
}

#[tokio::test]
async fn test_firefox_client() {
    env_logger::builder().is_test(true).try_init().ok();
    const PORT: u16 = 9995;
    let mut caps = serde_json::Map::new();
    caps.insert(
        "moz:firefoxOptions".to_string(),
        serde_json::json!({ "args": vec!["-headless"] }),
    );
    let mut c = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:4444")
        .await
        .unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    standup_server(rx, PORT).await;
    run_browser(&mut c, PORT).await;
    tx.send(()).unwrap();
}

#[tokio::test]
async fn test_chrome_client() {
    env_logger::builder().is_test(true).try_init().ok();
    const PORT: u16 = 9994;
    let mut caps = serde_json::Map::new();
    caps.insert(
        "goog:chromeOptions".to_string(),
        serde_json::json!({ "args": vec!["--headless"] }),
    );
    let mut c = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:9515")
        .await
        .unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    standup_server(rx, PORT).await;
    run_browser(&mut c, PORT).await;
    tx.send(()).unwrap();
}
