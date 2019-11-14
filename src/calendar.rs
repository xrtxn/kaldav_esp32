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
    fn new<S>(url: S) -> Self where S: Into<String> {
        Calendar {
            url: url.into(),
            auth: None,
        }
    }
}

impl Calendar {
    pub fn events(&self) -> crate::Result<Vec<crate::Event>> {
        let response = self.request("VEVENT");

        match response {
            Ok(response) => Ok(self.to_vec(response.as_str(), "//d:response/d:href/text()")),
            Err(err) => Err(err),
        }
    }

    pub fn tasks(&self) -> crate::Result<Vec<crate::Todo>> {
        let response = self.request("VTODO");

        match response {
            Ok(response) => Ok(self.to_vec(response.as_str(), "//d:response/d:href/text()")),
            Err(err) => Err(err),
        }
    }

    fn request(&self, filter: &str) -> crate::Result<String> {
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
