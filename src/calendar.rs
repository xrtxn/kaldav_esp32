use crate::Children;
use crate::Requestable;

#[derive(Clone, Debug)]
pub struct Calendar {
    url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Calendar {
    fn get_auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Calendar {
    fn get_url(&self) -> String {
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
        let date_format = "%Y%m%dT%H%M%S";

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
