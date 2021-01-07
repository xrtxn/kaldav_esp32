use crate::Requestable;

pub type Todo = Event;

#[derive(Clone, Debug)]
pub struct Event {
    pub url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Event {
    fn get_auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Event {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl crate::Children for Event {
    fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Event {
            url: url.into(),
            auth: None,
        }
    }
}

impl Event {
    pub fn attr(&self) -> crate::Result<ical::parser::ical::component::IcalCalendar> {
        match self.get(self.url.clone()) {
            Ok(calendar) => {
                let mut parser = ical::IcalParser::new(calendar.as_bytes());
                let event = parser.next().unwrap()?;

                Ok(event)
            },
            Err(err) => Err(crate::Error::Misc(format!("{:?}", err))),
        }
    }
}
