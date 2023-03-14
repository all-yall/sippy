use std::fmt::Display;

type BoxedError = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, BoxedError>;

#[derive(Debug)]
struct SimpleError {
    message: String
}


impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SimpleError {}


pub trait Contextable<T> {
    fn add_context(self, context: &'static str) -> Result<T>;
}

impl<R, E: Into<BoxedError>> Contextable<R> for std::result::Result<R, E> {
    fn add_context(self, context: &'static str) -> Result<R> {
        match self {
            Err(e) => {
                let message = format!("{} {}", context, &e.into());
                Err(Box::new(SimpleError{ message }))
            },
            Ok(r) => Ok(r)
        }
    }
}
