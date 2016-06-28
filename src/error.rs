use slack;
use cron;
use std::fmt;

pub enum Error {
    Slack(slack::Error),
    Cron(cron::error::CronParseError),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Slack(ref e) => write!(f, "Slack Error: {:?}", e),
            Error::Cron(ref e) => write!(f, "Cron Error: {:?}", e),
        }
    }
}
