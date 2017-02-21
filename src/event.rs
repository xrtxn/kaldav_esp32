use ::Requestable;
use ::std::collections::HashMap;

pub type Todo = Event;

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    url: String,
    auth: Option<::Authorization>,
    attr: HashMap<String, String>,
}

impl ::Requestable for Event {
    fn get_auth(&self) -> Option<::Authorization> {
        self.auth.clone()
    }

    fn set_raw_auth(&mut self, auth: Option<::Authorization>) {
        self.auth = auth;
    }
}

impl ::Xmlable for Event {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl ::Children for Event {
    fn new<S>(url: S) -> Self where S: Into<String> {
        let mut event = Event {
            url: url.into(),
            auth: None,
            attr: HashMap::new(),
        };

        event.attr = event.get_attributes();

        event
    }
}

impl Event {
    fn get_attributes(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
