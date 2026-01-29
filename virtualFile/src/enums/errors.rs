use std::{any::Any, error::Error, fmt::{self, Debug, Display}};

use wasmtime::Store;





#[derive(Debug)]
pub enum GlobalError {
    ParseError(String),
    UninitializedVariable,
    ResetOnceLock
}

impl fmt::Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            GlobalError::ParseError(msg) => msg,
            GlobalError::UninitializedVariable =>  "'tries to be waited for before being initialized.",
            GlobalError::ResetOnceLock => "Can't reset a oncelock static "
        };
        f.write_str(description)
    }
}

impl Error for GlobalError {}