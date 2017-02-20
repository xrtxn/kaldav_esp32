use hyper::header::Headers;
use std::convert::Into;
use std::io::prelude::*;

pub struct Caldav {
    url: String,
    http: ::hyper::client::Client,
    auth: Option<::hyper::header::Authorization<::hyper::header::Basic>>,
}

impl Caldav {
    pub fn new<S>(url: S) -> Self where S: Into<String> {
        let ssl = ::hyper_native_tls::NativeTlsClient::new().unwrap();
        let connector = ::hyper::net::HttpsConnector::new(ssl);
        let client = ::hyper::client::Client::with_connector(connector);

        Caldav {
            url: url.into(),
            http: client,
            auth: None,
        }
    }

    pub fn set_auth<S>(&mut self, username: S, password: Option<S>) where S: Into<String> {
        let password = match password {
            Some(password) => Some(password.into()),
            None => None,
        };

        self.auth = Some(
            ::hyper::header::Authorization(
                ::hyper::header::Basic {
                    username: username.into(),
                    password: password,
                }
            )
        );
    }

    pub fn principals(&self) -> ::result::Result<String> {
        self.propfind("/", r#"
<d:propfind xmlns:d="DAV:">
    <d:prop>
        <d:current-user-principal />
    </d:prop>
</d:propfind>
"#)
    }

    pub fn calendars<S>(&self, href: S) -> ::result::Result<String> where S: Into<String> {
        self.propfind(href, r#"
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:resourcetype />
     <d:displayname />
     <cs:getctag />
     <c:supported-calendar-component-set />
  </d:prop>
</d:propfind>
"#)
    }

    pub fn events<S>(&self, href: S) -> ::result::Result<String> where S: Into<String> {
        self.propfind(href, r#"
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:resourcetype />
  </d:prop>
</d:propfind>
"#)
    }

    pub fn tasks<S>(&self, href: S) -> ::result::Result<String> where S: Into<String> {
        let mut headers = ::hyper::header::Headers::new();

        headers.set_raw("Depth", vec![b"1".to_vec()]);

        self.report(href, r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:getetag />
        <c:calendar-data />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            <c:comp-filter name="VTODO" />
        </c:comp-filter>
    </c:filter>
</c:calendar-query>
"#, Some(headers))
    }

    pub fn event<S>(&self, href: S) -> ::result::Result<String> where S: Into<String> {
        self.get(href)
    }

    fn get<S>(&self, href: S) -> ::result::Result<String> where S: Into<String> {
        self.request("GET", href, None, None)
    }

    fn propfind<S>(&self, href: S, body: &str) -> ::result::Result<String> where S: Into<String> {
        self.request("PROPFIND", href, Some(body), None)
    }

    fn report<S>(&self, href: S, body: &str, headers: Option<Headers>) -> ::result::Result<String> where S: Into<String> {
        self.request("REPORT", href, Some(body), headers)
    }

    fn request<S>(&self, method: &str, href: S, body: Option<&str>, headers: Option<Headers>) -> ::result::Result<String> where S: Into<String> {
        let mut content = String::new();
        let method = ::hyper::method::Method::Extension(method.into());

        let mut headers = match headers {
            Some(headers) => headers,
            None =>  ::hyper::header::Headers::new(),
        };

        if let Some(auth) = self.auth.clone() {
            headers.set(auth);
        }

        let url = format!("{}/{}", &self.url, href.into());
        let mut request = self.http.request(method, &url)
            .headers(headers);

        if let Some(body) = body {
            request = request.body(body);
        }

        let mut response = request.send()?;

        match response.status {
            ::hyper::status::StatusCode::MultiStatus | ::hyper::status::StatusCode::Ok => {
                response.read_to_string(&mut content)?;
                Ok(content)
            },
            _ => Err(::result::Error::new(format!("{}", response.status))),
        }
    }
}
