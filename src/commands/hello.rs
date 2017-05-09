use regex::Regex;
use slack;
use error;
use brain::{Command, Disposition};

pub struct Hello {
    pattern: Regex,
}

impl Hello {
    pub fn new() -> Hello {
        Hello { pattern: Regex::new(r"(?i)\bhi\b|\bhello\b").unwrap() }
    }
}

impl Command for Hello {
    fn handle(&mut self,
              sender: &slack::Sender,
              text: &str,
              _: &str,
              channel: &str)
              -> Result<Disposition, error::Error> {
        if self.pattern.is_match(text) {
            sender.send_message(channel, "Hello to you")?;
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, _: &slack::Sender) {}

    fn usage(&self) -> &'static str {
        "`hi`/`hello`"
    }

    fn description(&self) -> &'static str {
        "Says hi to me, to which I reply"
    }
}
