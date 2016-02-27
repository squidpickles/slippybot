use slack;
use how_are_you::HowAreYou;
use hello::Hello;

pub enum Disposition {
    Handled,
    Unhandled,
    Terminal,
}

pub trait Command {
    fn handle(&mut self, cli: &mut slack::RtmClient, text: &str, user_id: &str, channel: &str) -> Result<Disposition, slack::Error>;
}

pub struct SlippyBrain {
    commands: Vec<Box<Command>>,
}

impl SlippyBrain {
    pub fn new() -> SlippyBrain {
        SlippyBrain {
            commands: vec![
                Box::new(Hello::new()),
                Box::new(HowAreYou::new()),
            ],
        }
    }

    pub fn interpret(&mut self, cli: &mut slack::RtmClient, text: &str, user_id: &str, channel: &str) -> Result<(), slack::Error> {
        let mut handled = false;
        for command in self.commands.iter_mut() {
            match try!(command.handle(cli, text, user_id, channel)) {
                Disposition::Handled => handled = true,
                Disposition::Unhandled => {},
                Disposition::Terminal => return Ok(()),
            }
        }
        if !handled {
            try!(cli.send_message(channel, "Can you try speaking Horse?"));
        }
        Ok(())
    }

    pub fn im(&self, cli: &mut slack::RtmClient, text: &str, user_id: &str) -> Result<(), slack::Error> {
        let im = try!(cli.im_open(user_id));
        try!(cli.send_message(&im.channel.id, text));
        Ok(())
    }
}
