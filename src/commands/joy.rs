use slack;
use error;
use brain::{Command, Disposition};
use cron::CronSchedule;
use chrono::{UTC, Local, Duration, TimeZone};
use chrono::datetime::DateTime;
use regex::Regex;
use rustc_serialize::json;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use rand::{self, Rng};

const NOTIFY_SCHEDULE: &'static str = "0 20 1-31 1-12 2 2000-3000"; // time is UTC
const NOTIFY_ROOM: &'static str = "#general";
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
    now_pattern: Regex,
    schedule: CronSchedule,
    last: Option<DateTime<UTC>>,
    list: JoyList,
}

impl Joy {
    pub fn new(enabled_on_startup: bool) -> Joy {
        let joy_list = JoyList::load(JOY_LIST_FILE).unwrap();
        let last = match enabled_on_startup {
            true => Some(UTC::now() - Duration::minutes(1)),
            false => None
        };
        Joy {
            start_pattern: Regex::new(r"(?i)start joy").unwrap(),
            stop_pattern: Regex::new(r"(?i)stop joy").unwrap(),
            now_pattern: Regex::new(r"(?i)joy now").unwrap(),
            schedule: CronSchedule::parse(NOTIFY_SCHEDULE).unwrap(),
            last: last,
            list: joy_list,
        }
    }

    pub fn start(&mut self) {
        self.last = Some(UTC::now() - Duration::minutes(1)); // cron scheduler always adds a minute for some reason
    }

    pub fn stop(&mut self) {
        self.last = None;
    }

    fn send_joy(&self, cli: &mut slack::RtmClient, channel: &str) -> Result<(), error::Error> {
        let chosen = self.list.choose();
        info!("Posting joy: {}", chosen);
        let message = format!("{}{}", JOY_PREFIX, chosen);
        try!(cli.send_message(channel, &message));
        Ok(())
    }

}

impl Command for Joy {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, _: &str, channel: &str) -> Result<Disposition, error::Error> {
        if self.start_pattern.is_match(text) {
            match self.last {
                Some(ref last) => {
                    let next_announcement = match self.schedule.next_utc_after(last) {
                        Some(next) => {
                            let local = Local.from_utc_datetime(&next.naive_local());
                            local.format("%A, %B %e, at %l:%M %p").to_string()
                        },
                        None => "Hmm, looks like never".to_string(),
                    };
                    try!(cli.send_message(channel, &format!("I'm already spouting joy. (You'll next hear from me on *{}*)", next_announcement)));
                    Ok(Disposition::Handled)
                },
                None => {
                    self.start();
                    try!(cli.send_message(channel, "I won't let you down."));
                    Ok(Disposition::Handled)
                }
            }
        } else if self.stop_pattern.is_match(text) {
            match self.last {
                Some(_) => {
                    self.stop();
                    try!(cli.send_message(channel, "I'll stop saying stuff."));
                    Ok(Disposition::Handled)
                },
                None => {
                    try!(cli.send_message(channel, "I'm already keeping quiet."));
                    Ok(Disposition::Handled)
                }
            }
        } else if self.now_pattern.is_match(text) {
            try!(self.send_joy(cli, channel));
            Ok(Disposition::Handled)
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
                            self.send_joy(cli, NOTIFY_ROOM).unwrap_or_else(|err| {
                                error!("Error sending joy: {}", err);
                                ()
                            });
                            self.last = Some(next);
                        } else {
                            debug!("Waiting until ready ({} -> {})", now, next);
                        }
                    },
                    None => {
                        warn!("No more dates. This should never happen!");
                        self.stop();
                    }
                }
            }
        }
    }

    fn usage(&self) -> &'static str {
        "`start`/`stop` `joy` or `joy now`"
    }

    fn description(&self) -> &'static str {
        "Starts or stops me announcing ways to preserve the joy. (Or says one right now)"
    }
}
