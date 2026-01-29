use std::{fmt,error::Error};


#[derive(Debug)]
pub enum GlobalHandlerError {
    // NotTerminal,
    TryFromIntError,
    // NotHandle(String)
}

impl fmt::Display for GlobalHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            // GlobalHandlerError::NotTerminal => "redirection are inpossible in this context ",
            GlobalHandlerError::TryFromIntError => "value can't be transform",
            // GlobalHandlerError::NotHandle(value) => &("action not handle for case ".to_string() + value)
        };
        f.write_str(description)
    }
}

impl Error for GlobalHandlerError {}

impl From<std::num::TryFromIntError> for GlobalHandlerError {
    fn from(_: std::num::TryFromIntError) -> Self {
        GlobalHandlerError::TryFromIntError
    }
}

