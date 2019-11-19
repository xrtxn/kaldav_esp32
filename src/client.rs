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
            &self.url,
            r#"
<d:propfind xmlns:d="DAV:">
    <d:prop>
        <d:current-user-principal />
    </d:prop>
</d:propfind>
"#,
        )?;

        Ok(self.to_vec(&response, "//d:current-user-principal/d:href/text()"))
    }

    fn principal(&self) -> crate::Result<crate::Principal> {
        let principals = self.principals()?;

        Ok(principals[0].clone())
    }

    fn home(&self) -> crate::Result<Vec<crate::Home>> {
        self.principal()?.home()
    }

    pub fn calendars(&self) -> crate::Result<HashMap<String, crate::Calendar>> {
        let home = self.home()?;

        home[0].calendars()
    }
}
