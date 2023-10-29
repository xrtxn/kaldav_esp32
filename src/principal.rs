use crate::Children;
use crate::Requestable;
use std::convert::Into;

#[derive(Clone, Debug)]
pub struct Principal {
    url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Principal {
    fn auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Principal {
    fn url(&self) -> String {
        self.url.clone()
    }
}

impl crate::Children for Principal {
    fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            auth: None,
        }
    }
}

impl Principal {
    pub fn home(&self) -> crate::Result<Vec<crate::Home>> {
        let response = self.propfind(
            &self.url,
            r#"
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:displayname />
     <c:calendar-home-set />
  </d:prop>
</d:propfind>
"#,
        )?;

        Ok(self.to_vec(&response, "//cal:calendar-home-set/d:href/text()"))
    }
}
