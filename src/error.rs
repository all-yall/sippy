use std::fmt::Display;

#[derive(Debug)]
struct SimpleError {
    message: String
}
impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for SimpleError { }

type BoxedError = Box<dyn std::error::Error>;
pub fn add_context<E: Into<BoxedError>>(context: &'static str) -> Box<dyn FnOnce(E) -> BoxedError> {
    Box::new(move |err: E| {
        let message = format!("{} {}", context, &err.into());
        Box::new(SimpleError {message})
    })
}

pub type Result<T> = std::result::Result<T, BoxedError>;

