use Requestable;
use std::convert::TryInto;

pub type Todo = Event;

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub url: String,
    auth: Option<::Authorization>,
}

impl ::Requestable for Event {
    fn get_auth(&self) -> Option<::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<::Authorization>) {
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
        Event {
            url: url.into(),
            auth: None,
        }
    }
}

impl Event {
    pub fn attr(&self) -> Result<::ical_parser::Content, String> {
        let calendar: Result<::ical_parser::VCalendar, String> = match self.get(self.url.clone()) {
            Ok(calendar) => calendar.try_into(),
            Err(err) => Err(format!("{:?}", err)),
        };

        match calendar {
            Ok(calendar) => Ok(calendar.content),
            Err(err) => Err(err),
        }
    }
}
