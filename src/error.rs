use slack;
use cron;
use serde_json;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    General(String),
    Slack(slack::Error),
    Cron(cron::error::CronParseError),
    Io(io::Error),
    Json(serde_json::Error),
}

/* TODO: lots of repetition in this file, eh? Do we resort to error-chain?
macro_rules! import_errors {
    $(($remote:ty, $local:ty),*) => {
        $(
        impl From<$remote> for Error {
            fn from(err: $remote) -> Error {
                $local(err)
            }
        }
        )*
    }
}
*/

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::General(err)
    }
}

impl From<slack::Error> for Error {
    fn from(err: slack::Error) -> Error {
        Error::Slack(err)
    }
}

impl From<cron::error::CronParseError> for Error {
    fn from(err: cron::error::CronParseError) -> Error {
        Error::Cron(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::General(ref e) => write!(f, "Error: {:?}", e),
            Error::Slack(ref e) => write!(f, "Slack Error: {:?}", e),
            Error::Cron(ref e) => write!(f, "Cron Error: {:?}", e),
            Error::Io(ref e) => write!(f, "Io Error: {:?}", e),
            Error::Json(ref e) => write!(f, "Json Error: {:?}", e),
        }
    }
}
