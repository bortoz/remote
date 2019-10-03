use cpython;
use crypto::symmetriccipher;
use sqlite;
use ssh2;
use std::{fmt, io, string};

pub struct RemoteError {
    message: String,
}

impl RemoteError {
    pub fn new(message: String) -> RemoteError {
        RemoteError { message: message }
    }
}

impl fmt::Display for RemoteError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.message)
    }
}

macro_rules! add_remote_error {
    ($x:ty) => {
        impl From<$x> for RemoteError {
            fn from(error: $x) -> RemoteError {
                RemoteError {
                    message: format!("{}", error),
                }
            }
        }
    };
    ($x:ty, $f:expr) => {
        impl From<$x> for RemoteError {
            fn from(error: $x) -> RemoteError {
                RemoteError {
                    message: format!($f, error),
                }
            }
        }
    };
}

add_remote_error!(io::Error);
add_remote_error!(sqlite::Error);
add_remote_error!(ssh2::Error);
add_remote_error!(string::FromUtf8Error);
add_remote_error!(symmetriccipher::SymmetricCipherError, "{:?}");

impl From<cpython::PyErr> for RemoteError {
    fn from(error: cpython::PyErr) -> RemoteError {
        RemoteError {
            message: match error.pvalue {
                Some(v) => v.to_string(),
                None => "cpython Exception".to_string(),
            },
        }
    }
}
