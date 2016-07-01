use slack;
use error;
use how_are_you::HowAreYou;
use hello::Hello;
use thanks::Thanks;
use joy::Joy;
use regex::Regex;

#[allow(dead_code)]
pub enum Disposition {
    Handled,
    Unhandled,
    Terminal,
}

pub trait Command {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, user_id: &str, channel: &str) -> Result<Disposition, error::Error>;
    fn periodic(&mut self, cli: &mut slack::RtmClient);
    fn usage(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

pub struct SlippyBrain {
    commands: Vec<Box<Command>>,
    help_pattern: Regex,
}

impl SlippyBrain {
    pub fn new() -> SlippyBrain {
        SlippyBrain {
            commands: vec![
                Box::new(Hello::new()),
                Box::new(Thanks::new()),
                Box::new(HowAreYou::new()),
                Box::new(Joy::new(true)),
            ],
            help_pattern: Regex::new(r"(?i)^help\b").unwrap(),
        }
    }

    pub fn periodic(&mut self, cli: &mut slack::RtmClient) {
        for task in self.commands.iter_mut() {
            task.periodic(cli);
        }
    }

    pub fn interpret(&mut self, cli: &mut slack::RtmClient, text: &str, user_id: &str, channel: &str) -> Result<(), error::Error> {
        let mut handled = false;
        // check for help command
        if self.help_pattern.is_match(text) {
            let mut help_message = "Things I understand:\n*`help`* - _Prints this message_".to_string();
            for command in self.commands.iter() {
                help_message.push_str(&format!("\n*{}* - _{}_", command.usage(), command.description()));
            }
            try!(cli.send_message(channel, &help_message));
            handled = true;
        }
        for command in self.commands.iter_mut() {
            match try!(command.handle(cli, text, user_id, channel)) {
                Disposition::Handled => handled = true,
                Disposition::Unhandled => {},
                Disposition::Terminal => return Ok(()),
            }
        }
        if !handled {
            try!(cli.send_message(channel, "Can you try speaking Horse? (You can ask for _help_)"));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn im(&self, cli: &mut slack::RtmClient, text: &str, user_id: &str) -> Result<(), error::Error> {
        let im = try!(cli.im_open(user_id));
        try!(cli.send_message(&im.channel.id, text));
        Ok(())
    }
}
