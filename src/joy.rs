use slack;
use error;
use brain::{Command, Disposition};
use cron::CronSchedule;
use chrono::{UTC, Duration};
use chrono::datetime::DateTime;
use regex::Regex;
use rustc_serialize::json;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use rand::{self, Rng};

const NOTIFY_SCHEDULE: &'static str = "0-59 0-23 1-31 1-12 0-7 2000-3000";
const NOTIFY_ROOM: &'static str = "#slippybottest";
const JOY_LIST_FILE: &'static str = "joy.json";
const JOY_PREFIX: &'static str = "Slippy says: ";

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct JoyList {
    items: Vec<String>,
}

impl JoyList {
    fn load<P: AsRef<Path>>(path: P) -> Result<JoyList, error::Error> {
        let mut file = try!(File::open(path));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        Ok(try!(json::decode(&data)))
    }

    fn choose(&self) -> &str {
        rand::thread_rng().choose(&self.items).unwrap()
    }
}

pub struct Joy {
    start_pattern: Regex,
    stop_pattern: Regex,
    schedule: CronSchedule,
    last: Option<DateTime<UTC>>,
    list: JoyList,
}

impl Joy {
    pub fn new() -> Joy {
        let joy_list = JoyList::load(JOY_LIST_FILE).unwrap();
        Joy {
            start_pattern: Regex::new(r"(?i)joy start").unwrap(),
            stop_pattern: Regex::new(r"(?i)joy stop").unwrap(),
            schedule: CronSchedule::parse(NOTIFY_SCHEDULE).unwrap(),
            last: None,
            list: joy_list,
        }
    }
}

impl Command for Joy {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, _: &str, channel: &str) -> Result<Disposition, error::Error> {
        if self.start_pattern.is_match(text) {
            match self.last {
                Some(_) => {
                    try!(cli.send_message(channel, "I'm already spouting joy!"));
                    Ok(Disposition::Handled)
                },
                None => {
                    self.last = Some(UTC::now() - Duration::minutes(1)); // cron scheduler always adds a minute for some reason
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
                let now = UTC::now();
                let next = self.schedule.next_utc_after(&last);
                match next {
                    Some(next) => {
                        if next < now {
                            let chosen = self.list.choose();
                            info!("Posting joy: {}", chosen);
                            let message = format!("{}{}", JOY_PREFIX, chosen);
                            cli.send_message(NOTIFY_ROOM, &message).unwrap_or_else(|err| {
                                error!("Error posting joy to room: {}", err);
                                0
                            });
                            self.last = Some(next);
                        } else {
                            info!("Waiting until ready ({} -> {})", now, next);
                        }
                    },
                    None => {
                        debug!("No more dates; stopping");
                        self.last = None;
                    }
                }
            }
        }
    }
}
