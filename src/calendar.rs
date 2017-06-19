use ::Children;
use ::Requestable;

#[derive(Clone, Debug, PartialEq)]
pub struct Calendar {
    url: String,
    auth: Option<::Authorization>,
}

impl ::Requestable for Calendar {
    fn get_auth(&self) -> Option<::Authorization> {
        self.auth.clone()
    }

    fn set_raw_auth(&mut self, auth: Option<::Authorization>) {
        self.auth = auth;
    }
}

impl ::Xmlable for Calendar {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl ::Children for Calendar {
    fn new<S>(url: S) -> Self where S: Into<String> {
        Calendar {
            url: url.into(),
            auth: None,
        }
    }
}

impl Calendar {
    pub fn events(&self) -> ::result::Result<Vec<::ical_parser::Event>> {
        let response = self.request("VEVENT");

        match response {
            Ok(response) => Ok(self.to_vec(response.as_str(), "//d:response/d:href/text()")),
            Err(err) => Err(err),
        }
    }

    pub fn tasks(&self) -> ::result::Result<Vec<::ical_parser::Todo>> {
        let response = self.request("VTODO");

        match response {
            Ok(response) => Ok(self.to_vec(response.as_str(), "//d:response/d:href/text()")),
            Err(err) => Err(err),
        }
    }

    fn request(&self, filter: &str) -> ::result::Result<String> {
        let body = format!(r#"
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
"#, filter);

        self.report(self.url.clone(), body.as_str())
    }
}
