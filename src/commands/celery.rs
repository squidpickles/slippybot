use regex::Regex;
use slack;
use error;
use brain::{Command, Disposition};

pub struct CeleryMan {
    pattern: Regex,
}

impl CeleryMan {
    pub fn new() -> CeleryMan {
        CeleryMan { pattern: Regex::new(r"(?i)load up celery man").unwrap() }
    }
}

impl Command for CeleryMan {
    fn handle(&mut self,
              sender: &slack::Sender,
              text: &str,
              _: &str,
              channel: &str)
              -> Result<Disposition, error::Error> {
        if self.pattern.is_match(text) {
            sender.send_message(channel, ":celery_man:")?;
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, _: &slack::Sender) {}

    fn usage(&self) -> &'static str {
        "`load up Celery Man`"
    }

    fn description(&self) -> &'static str {
        "Loads up Celery Man, for your important work"
    }
}
