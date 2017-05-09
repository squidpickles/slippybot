use slack;
use cron;
use serde_json;

error_chain!{
    links {
        Cron(cron::error::Error, cron::error::ErrorKind);
    }
    foreign_links {
        Slack(slack::Error);
        Io(::std::io::Error);
        Json(serde_json::Error);
    }
}
