use crate::status::NotModified;
use rocket::Request;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};

struct EtagStamp;

impl<'o> Into<Header<'o>> for EtagStamp {
    fn into(self) -> Header<'o> {
        match option_env!("ETAG") {
            Some(stamp) => Header::new("ETag", stamp),
            None => Header::new("X-Etag", "not-set"),
        }
    }
}

#[derive(Responder)]
pub struct EmbedEtag<T> {
    inner: T,
    stamp: EtagStamp,
}

impl<T> EmbedEtag<T> {
    pub fn new(inner: T) -> Self {
        EmbedEtag {
            inner,
            stamp: EtagStamp,
        }
    }
}

pub struct EtagCheck;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EtagCheck {
    type Error = NotModified;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let etag = match option_env!("ETAG") {
            None => {
                return Outcome::Success(EtagCheck);
            }
            Some(etag) => etag,
        };

        match req.headers().get_one("If-None-Match") {
            None => Outcome::Success(EtagCheck),
            Some(req_etag) if req_etag == etag => {
                Outcome::Error((Status::NotModified, NotModified("Etag Matched".to_string())))
            }
            Some(_) => Outcome::Success(EtagCheck),
        }
    }
}
