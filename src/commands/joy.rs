use slack;
use error;
use brain::{Command, Disposition};
use cron::Schedule;
use chrono::{UTC, Local, Duration, TimeZone};
use chrono::datetime::DateTime;
use regex::Regex;
use serde_json;
use std::path::Path;
use std::fs::File;
use std::str::FromStr;
use std::collections::BTreeMap;
use rand::{self, Rng};

const NOTIFY_SCHEDULE: &'static str = "0 0 20 1,4,7,10,13,16,19,21,24,27,30 * * *"; // time is UTC
const NOTIFY_ROOM: &'static str = "#general";
const JOY_LIST_FILE: &'static str = "joy.json";
const JOY_PREFIX: &'static str = "Slippy says: ";

pub struct JoyList {
    items: BTreeMap<String, Vec<String>>,
}

impl JoyList {
    fn load<P: AsRef<Path>>(path: P) -> Result<JoyList, error::Error> {
        let file = File::open(path)?;
        let joy = serde_json::from_reader(file)?;
        Ok(JoyList { items: joy })
    }

    fn choose(&self) -> &str {
        let mut rng = rand::thread_rng();
        let section_idx = rng.gen_range(0, self.items.len());
        let section = self.items.values().nth(section_idx).unwrap();
        rng.choose(section).unwrap()
    }
}

pub struct Joy {
    start_pattern: Regex,
    stop_pattern: Regex,
    now_pattern: Regex,
    schedule: Schedule,
    last: Option<DateTime<UTC>>,
    list: JoyList,
}

impl Joy {
    pub fn new(enabled_on_startup: bool) -> Joy {
        let joy_list = JoyList::load(JOY_LIST_FILE).unwrap();
        let last = if enabled_on_startup {
            Some(UTC::now() - Duration::minutes(1))
        } else {
            None
        };
        Joy {
            start_pattern: Regex::new(r"(?i)start joy").unwrap(),
            stop_pattern: Regex::new(r"(?i)stop joy").unwrap(),
            now_pattern: Regex::new(r"(?i)joy now").unwrap(),
            schedule: Schedule::from_str(NOTIFY_SCHEDULE).unwrap(),
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

    fn send_joy(&self, sender: &slack::Sender, channel: &str) -> Result<(), error::Error> {
        let chosen = self.list.choose();
        info!("Posting joy: {}", chosen);
        let message = format!("{}{}", JOY_PREFIX, chosen);
        sender.send_message(channel, &message)?;
        Ok(())
    }
}

impl Command for Joy {
    fn handle(&mut self,
              sender: &slack::Sender,
              text: &str,
              _: &str,
              channel: &str)
              -> Result<Disposition, error::Error> {
        if self.start_pattern.is_match(text) {
            match self.last {
                Some(ref last) => {
                    let next_announcement = match self.schedule.after(last).next() {
                        Some(next) => {
                            let local = Local.from_utc_datetime(&next.naive_local());
                            local.format("%A, %B %e, at %l:%M %p").to_string()
                        }
                        None => "Hmm, looks like never".to_string(),
                    };
                    sender.send_message(channel, &format!("I'm already spouting joy. (You'll next hear from me on *{}*)", next_announcement))?;
                    Ok(Disposition::Handled)
                }
                None => {
                    self.start();
                    sender.send_message(channel, "I won't let you down.")?;
                    Ok(Disposition::Handled)
                }
            }
        } else if self.stop_pattern.is_match(text) {
            match self.last {
                Some(_) => {
                    self.stop();
                    sender.send_message(channel, "I'll stop saying stuff.")?;
                    Ok(Disposition::Handled)
                }
                None => {
                    sender
                        .send_message(channel, "I'm already keeping quiet.")?;
                    Ok(Disposition::Handled)
                }
            }
        } else if self.now_pattern.is_match(text) {
            self.send_joy(sender, channel)?;
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, sender: &slack::Sender) {
        match self.last {
            None => {
                debug!("Not running");
                return;
            }
            Some(last) => {
                let now = UTC::now();
                let mut schedule_iterator = self.schedule.after(&last);
                match schedule_iterator.next() {
                    Some(next) => {
                        if next < now {
                            self.send_joy(sender, NOTIFY_ROOM)
                                .unwrap_or_else(|err| {
                                                    error!("Error sending joy: {}", err);
                                                    ()
                                                });
                            self.last = Some(next);
                        } else {
                            debug!("Waiting until ready ({} -> {})", now, next);
                        }
                    }
                    None => {
                        warn!("No more dates. This should never happen!");
                        self.last = None;
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
