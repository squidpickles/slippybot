extern crate slack;
extern crate regex;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate cron;
extern crate chrono;
extern crate rustc_serialize;
extern crate rand;

mod config;
mod brain;
mod error;
mod commands;

use config::Configuration;
use regex::Regex;
use brain::SlippyBrain;
use rustc_serialize::json::Json;

const CONFIG_FILE: &'static str = "slippybot.conf";

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
        if let slack::Event::Message(ref msg) = *(event.unwrap()) {
            if let slack::Message::Standard { ref channel, ref text, ref user, .. } = *msg {
                if let Some(ref txt) = *text {
                    info!("Message: {}", txt);
                    debug!("{}", raw_json);
                    match Json::from_str(raw_json) {
                        Ok(raw_msg) => {
                            if let Some(raw_msg_object) = raw_msg.as_object() {
                                if raw_msg_object.get("reply_to").is_some() {
                                    return;
                                }
                            }
                        },
                        Err(err) => error!("Error parsing raw JSON message: {}", err),
                    }
                    if let Some(ref re) = self.me_finder {
                        if let Some(ref chan) = *channel {
                            if re.is_match(txt) || chan.starts_with('D') {
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
            }
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
                let regex_str = format!(r"(?i)(?:^|[^:]){}|@{}", my_name, my_id);
                self.me_finder = Some(Regex::new(&regex_str).unwrap());
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let mut handler = SlippyHandler::new();
    let config = Configuration::load(CONFIG_FILE).unwrap();
    let mut cli = slack::RtmClient::new(config.api_key());
    cli.login_and_run(&mut handler).unwrap();
}
