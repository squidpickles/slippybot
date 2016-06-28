use slack;
use error;
use brain::{Command, Disposition};
use cron::CronSchedule;
use chrono::UTC;
use chrono::datetime::DateTime;
use regex::Regex;

const NOTIFY_SCHEDULE: &'static str = "0-59 0-23 1-31 1-12 0-7 2000-3000";

pub struct Joy {
    start_pattern: Regex,
    stop_pattern: Regex,
    schedule: CronSchedule,
    last: Option<DateTime<UTC>>,
}

impl Joy {
    pub fn new() -> Joy {
        Joy {
            start_pattern: Regex::new(r"(?i)joy start").unwrap(),
            stop_pattern: Regex::new(r"(?i)joy stop").unwrap(),
            schedule: CronSchedule::parse(NOTIFY_SCHEDULE).unwrap(),
            last: None,
        }
    }

}

impl Command for Joy {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, _: &str, channel: &str) -> Result<Disposition, error::Error> {
        if self.start_pattern.is_match(text) {
            match self.last {
                Some(_) => {
                    try!(cli.send_message(channel, "I'm already full of joy!"));
                    Ok(Disposition::Handled)
                },
                None => {
                    self.last = Some(UTC::now());
                    try!(cli.send_message(channel, "I won't let you down."));
                    Ok(Disposition::Handled)
                }
            }
        } else if self.stop_pattern.is_match(text) {
            match self.last {
                Some(_) => {
                    self.last = None;
                    try!(cli.send_message(channel, "I'll stop saying stuff."));
                    Ok(Disposition::Handled)
                },
                None => {
                    try!(cli.send_message(channel, "I'm already keeping quiet."));
                    Ok(Disposition::Handled)
                }
            }
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, cli: &mut slack::RtmClient) {
        match self.last {
            None => {
                debug!("Not running");
                return;
            },
            Some(last) => {
                debug!("Running");
                let now = UTC::now();
                let next = self.schedule.next_utc_after(&last);
                debug!("Last and next set");
                match next {
                    Some(next) => {
                        debug!("Next exists");
                        if next < now {
                            info!("Triggered");
                            self.last = Some(next);
                        } else {
                            info!("Waiting until ready ({}, {})", next, now);
                        }
                    },
                    None => {
                        debug!("Next doesn't exist");
                        self.last = None;
                    }
                }
            }
        }
    }
}
