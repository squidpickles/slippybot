use regex::Regex;
use slack;
use error;
use brain::{Command, Disposition};

pub struct Say {
    pattern: Regex,
}

impl Say {
    pub fn new() -> Say {
        Say {
            pattern: Regex::new(r"(?i)say (?:in <[#@](?P<channel>\w+)\|\w+> )?(?P<message>.+)")
                .unwrap(),
        }
    }
}

impl Command for Say {
    fn handle(&mut self,
              sender: &slack::Sender,
              text: &str,
              _: &str,
              channel: &str)
              -> Result<Disposition, error::Error> {
        if let Some(caps) = self.pattern.captures(text) {
            if let Some(message) = caps.name("message") {
                let reply_channel = match caps.name("channel") {
                    Some(ref channel) => channel.as_str(),
                    None => channel,
                };
                sender.send_message(reply_channel, message.as_str())?;
                return Ok(Disposition::Handled);
            }
        }
        Ok(Disposition::Unhandled)
    }

    fn periodic(&mut self, _: &slack::Sender) {}


    fn usage(&self) -> &'static str {
        "`say (in #channel) message"
    }

    fn description(&self) -> &'static str {
        "Says something (optionally in another channel)"
    }
}
