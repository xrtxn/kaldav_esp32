use crate::Children;
use crate::Requestable;
use std::collections::HashMap;
use std::convert::Into;

pub struct Client {
    url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Requestable for Client {
    fn get_auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}

impl crate::Xmlable for Client {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl crate::Children for Client {
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

impl Client {
    pub fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            auth: None,
        }
    }

    pub fn principals(&self) -> crate::Result<Vec<crate::Principal>> {
        let response = self.propfind(
            self.url.clone(),
            r#"
<d:propfind xmlns:d="DAV:">
    <d:prop>
        <d:current-user-principal />
    </d:prop>
</d:propfind>
"#,
        );

        match response {
            Ok(response) => Ok(self.to_vec(
                response.as_str(),
                "//d:current-user-principal/d:href/text()",
            )),
            Err(err) => Err(err),
        }
    }

    fn principal(&self) -> crate::Result<crate::Principal> {
        match self.principals() {
            Ok(p) => Ok(p[0].clone()),
            Err(err) => Err(err),
        }
    }

    fn home(&self) -> crate::Result<Vec<crate::Home>> {
        match self.principal() {
            Ok(principal) => principal.home(),
            Err(err) => Err(err),
        }
    }

    pub fn calendars(&self) -> crate::Result<HashMap<String, crate::Calendar>> {
        match self.home() {
            Ok(home) => home[0].calendars(),
            Err(err) => Err(err),
        }
    }
}
