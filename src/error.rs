use slack;
use cron;
use std::fmt;
use std::io;
use rustc_serialize::json;

#[derive(Debug)]
pub enum Error {
    Slack(slack::Error),
    Cron(cron::error::CronParseError),
    Io(io::Error),
    Json(json::DecoderError),
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

impl From<json::DecoderError> for Error {
    fn from(err: json::DecoderError) -> Error {
        Error::Json(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Slack(ref e) => write!(f, "Slack Error: {:?}", e),
            Error::Cron(ref e) => write!(f, "Cron Error: {:?}", e),
            Error::Io(ref e) => write!(f, "Io Error: {:?}", e),
            Error::Json(ref e) => write!(f, "Json Error: {:?}", e),
        }
    }
}
