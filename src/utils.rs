use rocket::http::Header;

struct Stamp;

impl<'o> Into<Header<'o>> for Stamp {
    fn into(self) -> Header<'o> {
        match option_env!("LAST_MODIFIED_STAMP") {
            Some(stamp) => Header::new("Last-Modified", stamp),
            None => Header::new("X-Last-Modified", "not-set"),
        }
    }
}

#[derive(Responder)]
pub struct EmbedLastModified<T> {
    inner: T,
    stamp: Stamp,
}

impl<T> EmbedLastModified<T> {
    pub fn new(inner: T) -> Self {
        EmbedLastModified {
            inner,
            stamp: Stamp,
        }
    }
}
