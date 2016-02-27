use regex::Regex;
use slack;
use brain::{Command, Disposition};

pub struct HowAreYou {
    pattern: Regex,
}

impl HowAreYou {
    pub fn new() -> HowAreYou {
        HowAreYou {
            pattern: Regex::new(r"(?i)how are you").unwrap(),
        }
    }
}

impl Command for HowAreYou {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, _: &str, channel: &str) -> Result<Disposition, slack::Error> {
        if self.pattern.is_match(text) {
            try!(cli.send_message(channel, "Great!"));
            Ok(Disposition::Handled)
        } else {
            Ok(Disposition::Unhandled)
        }
    }
}
