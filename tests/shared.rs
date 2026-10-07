use core::convert::Infallible;
use futures::Stream;
use rand::RngExt;
use regex_generate::DEFAULT_MAX_REPEAT;
use std::iter::FromIterator;
use warp::{sse::Event, Filter, Rejection, Reply};

pub fn sse(ct: u8) -> impl Stream<Item = Result<Event, Infallible>> {
    futures::stream::iter((0..=ct).into_iter().map({
        |i| {
            let mut rng = rand::rng();
            let data = String::from_iter((0..DEFAULT_MAX_REPEAT).map(|_| {
                let mut ch = '\n';
                while ch == '\n' || ch == '\r' {
                    ch = rng.random()
                }
                return ch;
            }));
            Ok(Event::default().id(i.to_string()).data(&data))
        }
    }))
}

pub fn sse_filter() -> impl Filter<Extract = (impl Reply,), Error = Rejection> {
    warp::path!("sse" / u8)
        .map(|ct: u8| warp::sse::reply(warp::sse::keep_alive().stream(sse(ct))).into_response())
}
