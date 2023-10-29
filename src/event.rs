use crate::Requestable;

pub type Todo = Event;

#[derive(Clone, Debug)]
pub struct Event {
    pub url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Event {
    fn auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Event {
    fn url(&self) -> String {
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
            }
            Err(err) => Err(crate::Error::Misc(format!("{:?}", err))),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn event() -> crate::Result {
        let server = crate::test::server();

        let client = crate::Client::new(server.url(""));
        let calendars = client.calendars()?;
        let calendar = calendars.get("Home calendar").unwrap();
        let events = calendar.events()?;
        let _event = events[0].attr()?;

        Ok(())
    }

    #[test]
    fn task() -> crate::Result {
        let server = crate::test::server();

        let client = crate::Client::new(server.url(""));
        let calendars = client.calendars()?;
        let calendar = calendars.get("My TODO list").unwrap();
        let tasks = calendar.tasks()?;
        let _task = tasks[0].attr()?;

        Ok(())
    }
}
