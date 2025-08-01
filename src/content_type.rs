#[derive(Responder)]
#[response(content_type = "image/x-icon")]
pub struct IcoFile<T>(pub T);
