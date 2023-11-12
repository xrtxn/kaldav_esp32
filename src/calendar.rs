use crate::Children;
use crate::Requestable;

#[derive(Clone, Debug, Default, crate::Object)]
pub struct Calendar {
    pub color: Option<String>,
    url: String,
    auth: Option<crate::Authorization>,
}

impl Calendar {
    pub fn objects(&self) -> crate::Result<crate::object::Iterator> {
        let response = self.request(None)?;

        Ok(
            crate::object::Iterator::from(
                self.to_vec(&response, "//d:response/d:href/text()")
            )
        )
    }

    pub fn events(&self) -> crate::Result<crate::object::Iterator> {
        let response = self.request(Some("VEVENT"))?;

        Ok(
            crate::object::Iterator::from(
                self.to_vec(&response, "//d:response/d:href/text()")
            )
        )
    }

    pub fn tasks(&self) -> crate::Result<crate::object::Iterator> {
        let response = self.request(Some("VTODO"))?;

        Ok(
            crate::object::Iterator::from(
                self.to_vec(&response, "//d:response/d:href/text()")
            )
        )
    }

    fn request(&self, filter: Option<&str>) -> crate::Result<String> {
        let filter = if let Some(filter) = filter {
            format!("<c:comp-filter name=\"{filter}\" />")
        } else {
            String::new()
        };
        let body = format!(
            r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:resourcetype />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            {filter}
        </c:comp-filter>
    </c:filter>
</c:calendar-query>
"#
        );

        self.report(&self.url, &body)
    }

    pub fn search<Tz>(
        &self,
        start: Option<chrono::DateTime<Tz>>,
        end: Option<chrono::DateTime<Tz>>,
    ) -> crate::Result<crate::object::Iterator>
    where
        Tz: chrono::TimeZone,
        Tz::Offset: std::fmt::Display,
    {
        let date_format = "%Y%m%dT%H%M%SZ";

        let start = start
            .map(|x| x.format(date_format).to_string())
            .unwrap_or_else(|| "-infinity".to_string());

        let end = end
            .map(|x| x.format(date_format).to_string())
            .unwrap_or_else(|| "+infinity".to_string());

        let body = format!(
            r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:resourcetype />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            <c:comp-filter name="VEVENT">
                <c:time-range start="{start}" end="{end}"/>
            </c:comp-filter>
        </c:comp-filter>
    </c:filter>
</c:calendar-query>"#
        );

        let response = self.report(&self.url, &body)?;

        Ok(
            crate::object::Iterator::from(
                self.to_vec(&response, "//d:response/d:href/text()")
            )
        )
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn calendar() -> crate::Result {
        let server = crate::test::server();

        let client = crate::Client::new(server.url(""));
        let calendars = client.calendars()?;
        let calendar = calendars.get("Home calendar").unwrap();
        assert_eq!(calendar.color.as_deref(), Some("#ffd4a5"));

        Ok(())
    }

    #[test]
    fn events() -> crate::Result {
        let server = crate::test::server();

        let client = crate::Client::new(server.url(""));
        let calendars = client.calendars()?;
        let calendar = calendars.get("Home calendar").unwrap();
        let events = calendar.events()?;
        assert!(!events.is_empty());

        Ok(())
    }

    #[test]
    fn search() -> crate::Result {
        let server = crate::test::server();

        let client = crate::Client::new(server.url(""));
        let calendars = client.calendars()?;
        let calendar = calendars.get("Home calendar").unwrap();
        let start = chrono::NaiveDate::from_ymd_opt(2023, 10, 28)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let events = calendar.search(Some(start), None)?;
        assert_eq!(events.len(), 1);

        Ok(())
    }
}
