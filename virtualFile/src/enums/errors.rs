use std::{error::Error, fmt::{self, Debug}};

#[derive(Debug)]
pub enum GlobalError {
    ParseError(String),
    UninitializedVariable,
    ResetOnceLock,
    TryFromIntError,
    NotExistingDir(String),
    JsonSerialize,
    FileToBig,
}

impl fmt::Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            GlobalError::ParseError(msg) => msg,
            GlobalError::UninitializedVariable =>  "'tries to be waited for before being initialized.",
            GlobalError::ResetOnceLock => "Can't reset a oncelock static ",
            GlobalError::TryFromIntError => "value can't be transform",
            GlobalError::JsonSerialize=> "can't serialize value",
            GlobalError::NotExistingDir(dir) => &("the dir:'".to_owned() + &dir + ""),
            GlobalError::FileToBig => "File to big to read"
        };
        f.write_str(description)
    }
}

impl Error for GlobalError {}


impl From<std::num::TryFromIntError> for GlobalError {
    fn from(_: std::num::TryFromIntError) -> Self {
        GlobalError::TryFromIntError
    }
}
