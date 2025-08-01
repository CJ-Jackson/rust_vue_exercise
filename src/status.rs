#[derive(Responder, Debug)]
#[response(status = 304)]
pub struct NotModified(pub String);
