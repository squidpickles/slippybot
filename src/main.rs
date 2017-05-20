extern crate slack;
extern crate regex;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate cron;
extern crate chrono;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate error_chain;

mod config;
mod brain;
mod error;
mod commands;

use config::Configuration;
use regex::Regex;
use brain::SlippyBrain;
use std::thread;
use std::time;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};

const CONFIG_FILE: &'static str = "slippybot.conf";

struct SlippyHandler {
    brain: Arc<Mutex<SlippyBrain>>,
    my_name: Option<String>,
    my_id: Option<String>,
    me_finder: Option<Regex>,
    interval: u64,
    running: Arc<AtomicBool>,
}

impl SlippyHandler {
    pub fn new(periodic_interval: u64) -> SlippyHandler {
        SlippyHandler {
            brain: Arc::new(Mutex::new(SlippyBrain::new())),
            my_name: None,
            my_id: None,
            me_finder: None,
            interval: periodic_interval,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    fn start_periodic(&mut self, sender: &slack::Sender) {
        let wait_time = time::Duration::from_secs(self.interval);
        let my_sender = sender.clone();
        let running = self.running.clone();
        let brain = self.brain.clone();
        thread::spawn(move || while running.load(Ordering::Relaxed) {
                          {
                              brain.lock().unwrap().periodic(&my_sender); // FIXME: unwrap
                          }
                          thread::sleep(wait_time);
                      });
    }
}

impl slack::EventHandler for SlippyHandler {
    fn on_event(&mut self, cli: &slack::RtmClient, event: slack::Event) {
        debug!("on_event()");
        if let slack::Event::Message(event_msg) = event {
            if let slack::Message::Standard(msg) = *event_msg {
                if let Some(ref txt) = msg.text {
                    info!("Message: {}", txt);
                    if let Some(ref my_id) = self.my_id {
                        if let Some(ref re) = self.me_finder {
                            if let Some(ref chan) = msg.channel {
                                if re.is_match(txt) || chan.starts_with('D') {
                                    if let Some(ref user_id) = msg.user {
                                        // Ignore our own messages
                                        if user_id != my_id {
                                            match self.brain
                                              .lock()
                                              .unwrap() // FIXME: unwrap
                                              .interpret(cli.sender(), txt, user_id, chan) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    error!("Error interpreting message: {}", err)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_close(&mut self, _: &slack::RtmClient) {
        debug!("on_close()");
        self.running.store(false, Ordering::Relaxed);
    }

    fn on_connect(&mut self, cli: &slack::RtmClient) {
        let start = cli.start_response();
        if let Some(ref user) = start.slf {
            if let Some(ref name) = user.name {
                if let Some(ref id) = user.id {
                    self.my_name = Some(name.clone());
                    self.my_id = Some(id.clone());
                    let regex_str = format!(r"(?i)(?:^|[^:]){}|@{}", name, id);
                    self.me_finder = Some(Regex::new(&regex_str).unwrap());
                }
            }
        }
        self.start_periodic(cli.sender());
        info!("Connected");
    }
}

fn main() {
    env_logger::init().unwrap();
    let config = Configuration::load(CONFIG_FILE).unwrap();
    let mut handler = SlippyHandler::new(config.interval);
    slack::RtmClient::login_and_run(&config.api_key, &mut handler).unwrap();
}
