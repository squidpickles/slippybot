use regex::Regex;
use slack;
use error;
use brain::{Command, Disposition};

pub struct Thanks {
    pattern: Regex,
}

impl Thanks {
    pub fn new() -> Thanks {
        Thanks { pattern: Regex::new(r"(?i)\bthanks\b|\bthank you\b").unwrap() }
    }
}

impl Command for Thanks {
    fn handle(&mut self,
              sender: &slack::Sender,
              text: &str,
              _: &str,
              channel: &str)
              -> Result<Disposition, error::Error> {
        if self.pattern.is_match(text) {
            sender.send_message(channel, "You're welcome!")?;
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, _: &slack::Sender) {}

    fn usage(&self) -> &'static str {
        "`thanks`/`thank you`"
    }

    fn description(&self) -> &'static str {
        "Thanks me for something I did. I'll accept, graciously."
    }
}
