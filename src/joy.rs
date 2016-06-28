use slack;
use error;
use brain::{Command, Disposition};
use cron::CronSchedule;
use chrono::UTC;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static WAIT_TIME: u64 = 1;

pub struct Joy {
    start_pattern: Regex,
    stop_pattern: Regex,
    running: Arc<Mutex<bool>>,
}

impl Joy {
    pub fn new() -> Joy {
        Joy {
            start_pattern: Regex::new(r"(?i)joy start (?P<schedule>\S+ \S+ \S+ \S+ \S+)").unwrap(),
            stop_pattern: Regex::new(r"(?i)joy stop").unwrap(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    fn start(&mut self, schedule: CronSchedule) -> Result<(), error::Error> {
        let running = self.running.clone();
        thread::spawn(move || {
            let wait_time = Duration::new(WAIT_TIME, 0);
            for next in schedule.upcoming().into_iter() {
                loop {
                    let now = UTC::now();
                    if now < next {
                        thread::sleep(wait_time);
                    } else {
                        // TODO: do something
                        break;
                    }
                    if !*running.lock().unwrap() {
                        return
                    }
                }
            }
        });
        Ok(())
    }

    fn stop(&mut self) {
        let running = self.running.clone();
        *running.lock().unwrap() = false;
    }

    fn running(&self) -> bool {
        let running = self.running.clone();
        return *running.lock().unwrap();
    }
}

impl Command for Joy {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, _: &str, channel: &str) -> Result<Disposition, error::Error> {
        if self.start_pattern.is_match(text) {
            if self.running() {
                try!(cli.send_message(channel, "The Preservation of Joy reminder is already running. Try running 'stop' first."));
                return Ok(Disposition::Handled)
            }
            let captures = self.start_pattern.captures(text).unwrap();
            let schedule_text = captures.name("schedule").unwrap();
            let schedule = try!(CronSchedule::parse(schedule_text));
            try!(self.start(schedule));
            try!(cli.send_message(channel, "Great!"));
            Ok(Disposition::Handled)
        } else if self.stop_pattern.is_match(text) {
            self.stop();
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }
}
