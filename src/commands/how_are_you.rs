use regex::Regex;
use slack;
use error;
use brain::{Command, Disposition};

pub struct HowAreYou {
    pattern: Regex,
}

impl HowAreYou {
    pub fn new() -> HowAreYou {
        HowAreYou { pattern: Regex::new(r"(?i)how are you").unwrap() }
    }
}

impl Command for HowAreYou {
    fn handle(&mut self,
              sender: &slack::Sender,
              text: &str,
              _: &str,
              channel: &str)
              -> Result<Disposition, error::Error> {
        if self.pattern.is_match(text) {
            sender.send_message(channel, "Great!")?;
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, _: &slack::Sender) {}

    fn usage(&self) -> &'static str {
        "`how are you`"
    }

    fn description(&self) -> &'static str {
        "Asks me how I'm doing, to which I'll reply"
    }
}
