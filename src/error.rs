
#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self { msg: e.to_string() }
    }
}
