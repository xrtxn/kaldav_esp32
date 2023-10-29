use crate::Requestable;

pub type Todo = Event;

#[derive(Clone, Debug, crate::Object)]
pub struct Event {
    pub url: String,
    auth: Option<crate::Authorization>,
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
