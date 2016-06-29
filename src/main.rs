extern crate slack;
extern crate regex;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate cron;
extern crate chrono;
extern crate rustc_serialize;
extern crate rand;
mod error;
mod brain;
mod hello;
mod how_are_you;
mod joy;

use regex::Regex;
use brain::SlippyBrain;
use rustc_serialize::json::Json;

const API_KEY: &'static str = "***REMOVED***";

struct SlippyHandler {
    brain: SlippyBrain,
    my_name: Option<String>,
    my_id: Option<String>,
    me_finder: Option<Regex>,
}

impl SlippyHandler {
    pub fn new() -> SlippyHandler {
        SlippyHandler {
            brain: SlippyBrain::new(),
            my_name: None,
            my_id: None,
            me_finder: None,
        }
    }
}

impl slack::EventHandler for SlippyHandler {
    fn on_event(&mut self,
                cli: &mut slack::RtmClient,
                event: Result<&slack::Event, slack::Error>,
                raw_json: &str) {
        debug!("on_event()");
        match *(event.unwrap()) {
            slack::Event::Message(ref msg) => {
                match *msg {
                    slack::Message::Standard { ref channel, ref text, ref user, ..  } => {
                        if let Some(ref txt) = *text {
                            info!("Message: {}", txt);
                            debug!("{}", raw_json);
                            match Json::from_str(raw_json) {
                                Ok(raw_msg) => {
                                    match raw_msg.as_object() {
                                        Some(raw_msg_object) => {
                                            match raw_msg_object.get("reply_to") {
                                                Some(_) => {
                                                    return; // re-sent message after disconnection; ignore
                                                },
                                                None => {}
                                            }
                                        },
                                        None => {}
                                    }
                                },
                                Err(err) => error!("Error parsing raw JSON message: {}", err),
                            }
                            if let Some(ref re) = self.me_finder {
                                if let Some(ref chan) = *channel {
                                    if re.is_match(txt) || chan.starts_with("D") {
                                        if let Some(ref user_id) = *user {
                                            match self.brain.interpret(cli, txt, user_id, chan) {
                                                Ok(_) => {},
                                                Err(err) => error!("Error interpreting message: {}", err),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    _ => {
                    }
                }
            }
            _ => {}
        }
    }

    fn on_ping(&mut self, cli: &mut slack::RtmClient) {
        debug!("on_ping()");
        self.brain.periodic(cli);
    }

    fn on_close(&mut self, _: &mut slack::RtmClient) {
        debug!("on_close()");
    }

    fn on_connect(&mut self, cli: &mut slack::RtmClient) {
        info!("Connected");
        self.my_name = cli.get_name();
        self.my_id = cli.get_id();
        if let Some(ref my_name) = self.my_name {
            if let Some(ref my_id) = self.my_id {
                let regex_str = format!(r"(?i)({})|(@{})", my_name, my_id);
                self.me_finder = Regex::new(&regex_str).ok();
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let mut handler = SlippyHandler::new();
    let mut cli = slack::RtmClient::new(&API_KEY);
    cli.login_and_run(&mut handler).unwrap();
}
