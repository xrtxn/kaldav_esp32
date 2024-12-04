use crate::Children;
use crate::Requestable;
use std::collections::BTreeMap;
use std::convert::Into;

#[derive(Debug, Default, crate::Object)]
pub struct Client {
    url: String,
    auth: Option<crate::Authorization>,
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
        let mut principals = self.principals()?;

        Ok(principals.remove(0))
    }

    fn home(&self) -> crate::Result<crate::Home> {
        self.principal()?.home()
    }

    pub fn calendars(&self) -> crate::Result<BTreeMap<String, crate::Calendar>> {
        let home = self.home()?;

        home.calendars()
    }

    pub fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        crate::Requestable::set_auth(self, auth)
    }
}
