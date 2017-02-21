use ::Children;
use ::Requestable;
use std::collections::HashMap;
use std::convert::Into;

pub struct Caldav {
    url: String,
    auth: Option<::hyper::header::Authorization<::hyper::header::Basic>>,
}

impl ::Requestable for Caldav {
    fn get_auth(&self) -> Option<::Authorization> {
        self.auth.clone()
    }

    fn set_raw_auth(&mut self, auth: Option<::Authorization>) {
        self.auth = auth;
    }
}

impl ::Xmlable for Caldav {
    fn get_url(&self) -> String {
        self.url.clone()
    }
}

impl ::Children for Caldav {
    fn new<S>(url: S) -> Self where S: Into<String> {
        Caldav {
            url: url.into(),
            auth: None,
        }
    }
}

impl Caldav {
    pub fn new<S>(url: S) -> Self where S: Into<String> {
        Caldav {
            url: url.into(),
            auth: None,
        }
    }

    pub fn principals(&self) -> ::result::Result<Vec<::principal::Principal>> {
        let response = self.propfind(self.url.clone(), r#"
<d:propfind xmlns:d="DAV:">
    <d:prop>
        <d:current-user-principal />
    </d:prop>
</d:propfind>
"#);

        match response {
            Ok(response) => Ok(self.to_vec(response.as_str(), "//d:current-user-principal/d:href/text()")),
            Err(err) => Err(err),
        }
    }

    fn principal(&self) -> ::result::Result<::principal::Principal> {
        match self.principals() {
            Ok(p) => Ok(p[0].clone()),
            Err(err) => Err(err),
        }
    }

    fn home(&self) -> ::result::Result<Vec<::home::Home>> {
        match self.principal() {
            Ok(principal) => principal.home(),
            Err(err) => Err(err),
        }
    }

    pub fn calendars(&self) -> ::result::Result<HashMap<String, ::calendar::Calendar>> {
        match self.home() {
            Ok(home) => home[0].calendars(),
            Err(err) => Err(err),
        }
    }
}
