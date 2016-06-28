use regex::Regex;
use slack;
use error;
use brain::{Command, Disposition};

pub struct Hello {
    pattern: Regex,
}

impl Hello {
    pub fn new() -> Hello {
        Hello {
            pattern: Regex::new(r"(?i)\bhi\b|\bhello\b").unwrap(),
        }
    }
}

impl Command for Hello {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, _: &str, channel: &str) -> Result<Disposition, error::Error> {
        if self.pattern.is_match(text) {
            try!(cli.send_message(channel, "Hello to you"));
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }

    fn periodic(&mut self, _: &mut slack::RtmClient) {
    }
}
