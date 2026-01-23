use fantoccini::{Client, ClientBuilder, Locator};
use warp::Filter;

mod shared;
use shared::*;

async fn standup_server(rx: tokio::sync::oneshot::Receiver<()>, port: u16) {
    let server = warp::serve(
        sse_filter()
            .or(warp::fs::dir("tests/browser_client_assets"))
            .boxed()
            .with(warp::log("test-sse-server")),
    )
    .bind(([127, 0, 0, 1], port))
    .await
    .graceful(async move {
        rx.await.ok();
    });

    tokio::task::spawn(server.run());
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
}

#[tokio::test]
async fn test_firefox_client() {
    env_logger::builder().is_test(true).try_init().ok();
    const PORT: u16 = 9995;
    let (tx, rx) = tokio::sync::oneshot::channel();
    standup_server(rx, PORT).await;
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

    run_browser(&mut c, PORT).await;
    c.close().await.unwrap();
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
    c.close().await.unwrap();
    tx.send(()).unwrap();
}
