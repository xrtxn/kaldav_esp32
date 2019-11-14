use crate::Children;
use crate::Requestable;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Home {
    url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Home {
    fn get_auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Home {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl crate::Children for Home {
    fn new<S>(url: S) -> Self where S: Into<String> {
        Home {
            url: url.into(),
            auth: None,
        }
    }
}

impl Home {
    pub fn calendars(&self) -> crate::Result<HashMap<String, crate::Calendar>> {
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
