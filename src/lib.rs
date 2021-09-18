use std::{borrow::Cow, iter::Peekable, str::Chars};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerSentEvent<'a> {
    pub comment: Option<Cow<'a, str>>,
    pub event: Option<Cow<'a, str>>,
    pub id: Option<Cow<'a, str>>,
    pub data: Option<Cow<'a, str>>,
    pub retry: Option<Cow<'a, str>>,
}

pub struct SSEBuilder<'a> {
    inner: ServerSentEvent<'a>,
}

impl<'a> ServerSentEvent<'a> {
    pub fn builder() -> SSEBuilder<'a> {
        SSEBuilder {
            inner: Self::default(),
        }
    }
}

impl<'a> SSEBuilder<'a> {
    pub fn comment(mut self, comment: &'a str) -> Self {
        if let Some(old) = self.inner.comment.replace(Cow::Borrowed(comment)) {
            log::warn!("overwriting comment: {:?} with {:?}", old, comment);
        }
        self
    }
    pub fn event(mut self, comment: &'a str) -> Self {
        if let Some(old) = self.inner.event.replace(Cow::Borrowed(comment)) {
            log::warn!("overwriting event: {:?} with {:?}", old, comment);
        }
        self
    }
    pub fn id(mut self, comment: &'a str) -> Self {
        if let Some(old) = self.inner.id.replace(Cow::Borrowed(comment)) {
            log::warn!("overwriting id: {:?} with {:?}", old, comment);
        }
        self
    }
    pub fn data(mut self, data: &'a str) -> Self {
        if let Some(old) = self.inner.data.replace(Cow::Borrowed(data)) {
            log::warn!("overwriting data: {:?} with {:?}", old, data);
        }
        self
    }
    pub fn retry(mut self, retry: &'a str) -> Self {
        if let Some(old) = self.inner.retry.replace(Cow::Borrowed(retry)) {
            log::warn!("overwriting retry: {:?} with {:?}", old, retry);
        }
        self
    }
    pub fn build(self) -> ServerSentEvent<'a> {
        self.inner
    }
}

impl<'a> std::fmt::Display for ServerSentEvent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(comment) = &self.comment {
            write!(f, ":{}\n", comment)?;
        }
        if let Some(event) = &self.event {
            write!(f, "event:{}\n", event)?;
        }
        if let Some(id) = &self.id {
            write!(f, "id:{}\n", id)?;
        }
        if let Some(retry) = &self.retry {
            write!(f, "retry:{}\n", retry)?;
        }
        if let Some(data) = &self.data {
            write!(f, "data:{}\n", data)?;
        }
        write!(f, "\n")
    }
}

pub struct Parser<'a> {
    orig: &'a str,
    chars: Peekable<Chars<'a>>,
    idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(orig: &'a str) -> Self {
        let orig = orig.trim_start_matches('\u{FEFF}').trim_start();
        Self {
            orig,
            chars: orig.chars().peekable(),
            idx: 0,
        }
    }

    pub fn next_event(&mut self) -> Option<ServerSentEvent<'a>> {
        if self.at_end() {
            return None;
        }
        let mut pending = ServerSentEvent::default();
        loop {
            let ty = self.eat_keyword()?;
            log::debug!("next field type: {:?}", ty);
            let start = self.seek_next_new_line()?;

            match ty {
                Keyword::Data => {
                    pending.data = Some(self.slice_backwards(start));
                }
                Keyword::Event => pending.event = Some(self.slice_backwards(start)),
                Keyword::Id => {
                    pending.id = Some(self.slice_backwards(start));
                }
                Keyword::Retry => {
                    pending.retry = Some(self.slice_backwards(start));
                }
                Keyword::Comment => {
                    pending.comment = Some(self.slice_backwards(start));
                }
                Keyword::Unknown(ref k) => {
                    log::warn!(
                        "dropping unknown field {}:{}",
                        k,
                        self.slice_backwards(start)
                    );
                }
            }
            if !self.eat_new_line() {
                log::warn!("No new line after field {:?}", ty);
            }
            if self.eat_new_line() {
                break;
            }
        }
        Some(pending)
    }

    /// Extract the provide length from the current index
    fn slice_backwards(&self, start: usize) -> Cow<'a, str> {
        log::debug!("slice_backwards from {}..{}", start, self.idx);
        Cow::Borrowed(&self.orig[start..self.idx])
    }

    /// Find the next unescaped new line character
    /// returning the true if a new line was found
    fn seek_next_new_line(&mut self) -> Option<usize> {
        let mut last_char_was_escape = false;
        let start = self.idx;
        while let Some(&ch) = self.chars.peek() {
            if ch == '\n' || ch == '\r' && !last_char_was_escape {
                return Some(start);
            }
            last_char_was_escape = ch == '\\';
            self.next_char();
        }
        None
    }

    /// consume an expected keyword
    fn eat_keyword(&mut self) -> Option<Keyword<'a>> {
        let start = self.idx;
        while let Some(&ch) = self.chars.peek() {
            if ch == ':' {
                let s = self.slice_backwards(start);
                self.next_char();
                return Some(s.into());
            }
            self.next_char();
        }
        None
    }

    fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            self.idx += ch.len_utf8();
            return Some(ch);
        }
        None
    }

    fn eat_new_line(&mut self) -> bool {
        if let Some(&next) = self.chars.peek() {
            if next == '\r' {
                self.next_char();
                if let Some(&'\n') = self.chars.peek() {
                    self.next_char();
                }
                return true;
            } else if next == '\n' {
                self.next_char();
                return true;
            }
        }
        false
    }

    fn at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}

#[derive(Debug, Clone)]
pub enum Keyword<'a> {
    Data,
    Event,
    Id,
    Retry,
    Comment,
    Unknown(Cow<'a, str>),
}

impl<'a> From<Cow<'a, str>> for Keyword<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        log::debug!("into keyword: {}", s);
        match s.as_ref() {
            "data" => Self::Data,
            "event" => Self::Event,
            "id" => Self::Id,
            "retry" => Self::Retry,
            "" => Self::Comment,
            _ => Self::Unknown(s),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn data_only_round_trip_empty() {
        env_logger::builder().is_test(true).try_init().ok();
        data_only_round_trip("");
    }

    proptest! {
        #[test]
        fn data_only_encoded_props(s in "[^\r\n]*") {
            let ev = ServerSentEvent::builder()
                .data(&s)
                .build();
            let encoded = format!("{}", ev);
            assert_eq!(encoded, format!("data:{}\n\n", s));
        }
        #[test]
        fn data_only_round_trip_props(s in "[^\r\n]*") {
            env_logger::builder().is_test(true).try_init().ok();
            data_only_round_trip(&s);
        }
        #[test]
        fn data_event_round_trip_props((data, event) in ("[^\r\n]*", "[^\r\n]*")) {
            env_logger::builder().is_test(true).try_init().ok();
            data_event_round_trip(&data, &event);
        }
        #[test]
        fn data_event_retry_round_trip_props((data, event, retry) in ("[^\r\n]*", "[^\r\n]*", "[^\r\n]*")) {
            env_logger::builder().is_test(true).try_init().ok();
            data_event_retry_round_trip(&data, &event, &retry);
        }
        #[test]
        fn data_event_retry_id_round_trip_props((data, event, retry, id) in ("[^\r\n]*", "[^\r\n]*", "[^\r\n]*", "[^\r\n]*")) {
            env_logger::builder().is_test(true).try_init().ok();
            data_event_retry_id_round_trip(&data, &event, &retry, &id);
        }
        #[test]
        fn all_round_trip_props((data, event, retry, id, comment) in ("[^\r\n]*", "[^\r\n]*", "[^\r\n]*", "[^\r\n]*", "[^\r\n]*")) {
            env_logger::builder().is_test(true).try_init().ok();
            all_round_trip(&data, &event, &retry, &id, &comment);
        }
    }

    fn data_only_round_trip(s: &str) {
        let ev = ServerSentEvent::builder().data(&s).build();
        let encoded = format!("{}", ev);
        let mut parser = Parser::new(&encoded);
        let ev2 = parser.next_event().unwrap();
        assert_eq!(parser.next_event(), None);
        assert_eq!(ev, ev2);
    }

    fn data_event_round_trip(data: &str, event: &str) {
        let ev = ServerSentEvent::builder().data(data).event(event).build();
        let encoded = format!("{}", ev);
        let mut parser = Parser::new(&encoded);
        let ev2 = parser.next_event().unwrap();
        assert_eq!(parser.next_event(), None);
        assert_eq!(ev, ev2);
    }
    fn data_event_retry_round_trip(data: &str, event: &str, retry: &str) {
        let ev = ServerSentEvent::builder()
            .data(data)
            .event(event)
            .retry(retry)
            .build();
        let encoded = format!("{}", ev);
        let mut parser = Parser::new(&encoded);
        let ev2 = parser.next_event().unwrap();
        assert_eq!(parser.next_event(), None);
        assert_eq!(ev, ev2);
    }
    fn data_event_retry_id_round_trip(data: &str, event: &str, retry: &str, id: &str) {
        let ev = ServerSentEvent::builder()
            .data(data)
            .event(event)
            .retry(retry)
            .id(id)
            .build();
        let encoded = format!("{}", ev);
        let mut parser = Parser::new(&encoded);
        let ev2 = parser.next_event().unwrap();
        assert_eq!(parser.next_event(), None);
        assert_eq!(ev, ev2);
    }
    fn all_round_trip(data: &str, event: &str, retry: &str, id: &str, comment: &str) {
        let ev = ServerSentEvent::builder()
            .data(data)
            .event(event)
            .retry(retry)
            .id(id)
            .comment(comment)
            .build();
        let encoded = format!("{}", ev);
        let mut parser = Parser::new(&encoded);
        let ev2 = parser.next_event().unwrap();
        assert_eq!(parser.next_event(), None);
        assert_eq!(ev, ev2);
    }
}
