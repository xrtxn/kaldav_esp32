use ::Children;
use ::Requestable;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Home {
    url: String,
    auth: Option<::Authorization>,
}

impl ::Requestable for Home {
    fn get_auth(&self) -> Option<::Authorization> {
        self.auth.clone()
    }

    fn set_raw_auth(&mut self, auth: Option<::Authorization>) {
        self.auth = auth;
    }
}

impl ::Xmlable for Home {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl ::Children for Home {
    fn new<S>(url: S) -> Self where S: Into<String> {
        Home {
            url: url.into(),
            auth: None,
        }
    }
}

impl Home {
    pub fn calendars(&self) -> ::result::Result<HashMap<String, ::calendar::Calendar>> {
        let response = self.propfind(self.url.clone(), r#"
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:resourcetype />
     <d:displayname />
     <cs:getctag />
     <c:supported-calendar-component-set />
  </d:prop>
</d:propfind>
"#);

        match response {
            Ok(response) => Ok(self.to_map(response.as_str(), "//d:response//d:displayname/text()", "//d:displayname [text() = '{}']/../../../d:href/text()")),
            Err(err) => Err(err),
        }
    }
}
