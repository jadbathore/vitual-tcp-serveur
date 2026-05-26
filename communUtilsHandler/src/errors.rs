use std::{error::Error, fmt::{self, Debug}};

use colored::Colorize;

#[derive(Debug)]
pub enum GlobalError {
    ParseError(String),
    UninitializedVariable,
    ResetOnceLock,
    TryFromIntError,
    NotExistingDir(String),
    JsonSerialize,
    FileToBig,
    WasiError,
    SingleInstanceBreach,
    StringEnumInit(String),
    IoError(String),
    InitError(String)
}

impl fmt::Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            GlobalError::UninitializedVariable =>  "'tries to be waited for before being initialized.",
            GlobalError::ResetOnceLock => "Can't reset a oncelock static.",
            GlobalError::TryFromIntError => "value can't be transform.",
            GlobalError::JsonSerialize=> "can't serialize value.",
            GlobalError::NotExistingDir(dir) => &("the dir:'".to_owned() + &dir + "."),
            GlobalError::FileToBig => "File to big to read.",
            GlobalError::WasiError => "Something went wrong during the runing of a wasi component.",
            GlobalError::SingleInstanceBreach => "instance cannot be duplicated.",
            GlobalError::StringEnumInit(variante) => &("variante enum: ".to_owned() + variante + "doesn't exist."),
            GlobalError::ParseError(string) | GlobalError::IoError(string) | GlobalError::InitError(string) => string,
            
        };
        f.write_str(&description.red().bold().to_string())
    }
}

impl Error for GlobalError {}

impl From<std::io::Error> for Box<GlobalError> {
    fn from(value: std::io::Error) -> Self {
        Box::new(GlobalError::IoError(value.kind().to_string()))
    }
}

impl From<std::num::TryFromIntError> for GlobalError {
    fn from(_: std::num::TryFromIntError) -> Self {
        GlobalError::TryFromIntError
    }
}
