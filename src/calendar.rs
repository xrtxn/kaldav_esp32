use crate::Children;
use crate::Requestable;

#[derive(Clone, Debug)]
pub struct Calendar {
    url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Calendar {
    fn auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Calendar {
    fn url(&self) -> String {
        self.url.clone()
    }
}

impl crate::Children for Calendar {
    fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Calendar {
            url: url.into(),
            auth: None,
        }
    }
}

impl Calendar {
    pub fn events(&self) -> crate::Result<Vec<crate::Event>> {
        let response = self.request("VEVENT")?;

        Ok(self.to_vec(&response, "//d:response/d:href/text()"))
    }

    pub fn tasks(&self) -> crate::Result<Vec<crate::Todo>> {
        let response = self.request("VTODO")?;

        Ok(self.to_vec(&response, "//d:response/d:href/text()"))
    }

    fn request(&self, filter: &str) -> crate::Result<String> {
        let body = format!(
            r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:resourcetype />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            <c:comp-filter name="{}" />
        </c:comp-filter>
    </c:filter>
</c:calendar-query>
"#,
            filter
        );

        self.report(&self.url, &body)
    }

    pub fn search<Tz>(
        &self,
        start: Option<chrono::DateTime<Tz>>,
        end: Option<chrono::DateTime<Tz>>,
    ) -> crate::Result<Vec<crate::Event>>
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

        Ok(self.to_vec(&response, "//d:response/d:href/text()"))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn events() -> crate::Result {
        let server = crate::test::server();

        let client = crate::Client::new(server.url(""));
        let calendars = client.calendars()?;
        let calendar = calendars.get("Home calendar").unwrap();
        let events = calendar.events()?;
        assert_eq!(events.len(), 1);

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
