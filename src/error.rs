use std::error;
use std::fmt;

#[derive(Debug)]
pub enum MudError {
    NoAddress,
    InvalidAddress,
    VerbNoClient,
    VerbNotFound,
    VerbnoClients,
}

impl fmt::Display for MudError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MudError::NoAddress => write!(f, "No address has been bound to the server."),
            MudError::InvalidAddress => write!(f, "The address is invalid."),
            MudError::VerbNoClient => write!(f, "No client provided to the verb."),
            MudError::VerbNotFound => write!(f, "The verb was not found."),
            MudError::VerbnoClients => write!(f, "No clients provided to the verb."),
        }
    }
}

impl error::Error for MudError {
    fn description(&self) -> &str {
        match *self {
            MudError::NoAddress => "No address has been bound to the server.",
            MudError::InvalidAddress => "The address is invalid.",
            MudError::VerbNoClient => "No client provided to the verb.",
            MudError::VerbNotFound => "The verb was not found.",
            MudError::VerbnoClients => "No clients provided to the verb.",
        }
    }
}
