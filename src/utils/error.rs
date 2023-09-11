use std::fmt;


#[derive(Debug)]
pub enum MyError {
    Reqwest(reqwest::Error),
    ErrorStr(String),
}
impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> Self {
        MyError::Reqwest(err)
    }
}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Reqwest(err) => write!(f, "Reqwest error: {}", err),
            MyError::ErrorStr(err) => write!(f, "ErrorStr: {}", err),
        }
    }
}
