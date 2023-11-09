mod calendar;
mod client;
mod event;
mod home;
mod principal;
mod result;

pub use client::*;
pub use calendar::*;
pub use event::*;
pub use home::*;
pub use principal::*;
pub use result::*;

use caldav_derive::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Authorization {
    pub username: String,
    pub password: Option<String>,
}

trait Requestable {
    fn auth(&self) -> Option<Authorization>;
    fn set_auth(&mut self, auth: Option<Authorization>);

    fn get<S>(&self, href: S) -> Result<String>
    where
        S: Into<String>,
    {
        self.request("GET", href, None, None)
    }

    fn propfind<S>(&self, href: S, body: &str) -> Result<String>
    where
        S: Into<String>,
    {
        self.request("PROPFIND", href, Some(body), None)
    }

    fn report<S>(&self, href: S, body: &str) -> Result<String>
    where
        S: Into<String>,
    {
        let mut headers = BTreeMap::new();

        headers.insert("Depth", "1");

        self.request("REPORT", href, Some(body), Some(headers))
    }

    fn request<S>(
        &self,
        method: &str,
        href: S,
        body: Option<&str>,
        headers: Option<BTreeMap<&'static str, &'static str>>,
    ) -> Result<String>
    where
        S: Into<String>,
    {
        let mut request = attohttpc::RequestBuilder::new(
            attohttpc::Method::from_bytes(method.as_bytes()).unwrap(),
            &href.into(),
        )
        .text(body.unwrap_or_default());

        if let Some(headers) = headers {
            for (key, value) in &headers {
                request = request.header(*key, *value);
            }
        }

        if let Some(auth) = self.auth() {
            request = request.basic_auth(auth.username, auth.password);
        }

        let response = request.send()?;

        if response.is_success() {
            Ok(response.text()?)
        } else {
            Err(Error::new(format!("{}", response.status())))
        }
    }
}

trait Xmlable {
    fn url(&self) -> &str;

    fn xml(xml: &str, xpath: &str) -> Vec<String> {
        let package = sxd_document::parser::parse(xml).unwrap();
        let document = package.as_document();
        let root = document.root().children()[0];

        let mut context = sxd_xpath::Context::new();
        context.set_namespace("d", "DAV:");
        context.set_namespace("cal", "urn:ietf:params:xml:ns:caldav");

        let factory = sxd_xpath::Factory::new();

        let xpath = factory
            .build(xpath)
            .expect("Could not compile XPath")
            .expect("No XPath was compiled");

        let nodes = xpath.evaluate(&context, root).unwrap();

        let mut results = vec![];

        if let sxd_xpath::Value::Nodeset(nodes) = nodes {
            for node in nodes.iter() {
                results.push(String::from(node.text().unwrap().text()));
            }
        }

        results
    }

    fn append_host(&self, href: String) -> String {
        let url = url::Url::parse(self.url()).unwrap();
        let port = url.port().map(|x| format!(":{x}")).unwrap_or_default();

        format!("{}://{}{port}{href}", url.scheme(), url.host_str().unwrap())
    }
}

trait Children: Requestable + Xmlable {
    fn new<S>(url: S) -> Self
    where
        S: Into<String>;

    fn to_vec<C>(&self, response: &str, xpath: &str) -> Vec<C>
    where
        C: Children + Requestable,
    {
        Self::xml(response, xpath)
            .iter()
            .map(|x| {
                let mut element = C::new(self.append_host(x.clone()));

                element.set_auth(self.auth());

                element
            })
            .collect()
    }

    fn to_map<C>(&self, response: &str, key_xpath: &str, value_xpath: &str) -> BTreeMap<String, C>
    where
        C: Children + Requestable,
    {
        let mut map = BTreeMap::new();
        let keys = Self::xml(response, key_xpath);

        for key in keys {
            let xpath = value_xpath.replace("{}", key.as_str());
            let values = Self::xml(response, xpath.as_str());

            let mut element = C::new(self.append_host(values[0].clone()));
            element.set_auth(self.auth());

            map.insert(key.to_string(), element);
        }

        map
    }
}

#[cfg(test)]
mod test {
    pub(crate) fn server() -> httpmock::MockServer {
        env_logger::try_init().ok();
        let server = httpmock::MockServer::start();

        server.mock(|when, then| {
            when.path("/")
                .body(r#"
<d:propfind xmlns:d="DAV:">
    <d:prop>
        <d:current-user-principal />
    </d:prop>
</d:propfind>
"#);
            then.status(207)
                .body(r#"
<d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/">
    <d:response>
        <d:href>/</d:href>
        <d:propstat>
            <d:prop>
                <d:current-user-principal>
                    <d:href>/principals/users/johndoe/</d:href>
                </d:current-user-principal>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>"#);
        });

        server.mock(|when, then| {
            when.path("/principals/users/johndoe/")
                .body(r#"
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:displayname />
     <c:calendar-home-set />
  </d:prop>
</d:propfind>
"#);

            then.status(207)
                .body(r#"
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:response>
        <d:href>/principals/users/johndoe/</d:href>
        <d:propstat>
            <d:prop>
                <c:calendar-home-set>
                    <d:href>/calendars/johndoe/</d:href>
                </c:calendar-home-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>
"#);
        });

        server.mock(|when, then| {
            when.path("/calendars/johndoe/")
                .body(r#"
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:resourcetype />
     <d:displayname />
     <cs:getctag />
     <c:supported-calendar-component-set />
  </d:prop>
</d:propfind>
"#);

            then.status(207)
                .body(r#"
<d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:response>
        <d:href>/calendars/johndoe/</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype>
                    <d:collection/>
                </d:resourcetype>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
    <d:response>
        <d:href>/calendars/johndoe/home/</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype>
                    <d:collection/>
                    <c:calendar/>
                </d:resourcetype>
                <d:displayname>Home calendar</d:displayname>
                <cs:getctag>3145</cs:getctag>
                <c:supported-calendar-component-set>
                    <c:comp name="VEVENT" />
                </c:supported-calendar-component-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
    <d:response>
        <d:href>/calendars/johndoe/tasks/</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype>
                    <d:collection/>
                    <c:calendar/>
                </d:resourcetype>
                <d:displayname>My TODO list</d:displayname>
                <cs:getctag>3345</cs:getctag>
                <c:supported-calendar-component-set>
                    <c:comp name="VTODO" />
                </c:supported-calendar-component-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>
"#);
        });

        server.mock(|when, then| {
            when.path("/calendars/johndoe/home/")
                .header("Depth", "1")
                .body(r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:resourcetype />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            <c:comp-filter name="VEVENT" />
        </c:comp-filter>
    </c:filter>
</c:calendar-query>
"#);

            then.status(207)
                .body(r#"
<d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:response>
        <d:href>/calendars/johndoe/home/132456-34365.ics</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype/>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>
"#);
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/calendars/johndoe/home/132456-34365.ics");

            then.status(200)
                .body(r#"
BEGIN:VCALENDAR
VERSION:2.0
CALSCALE:GREGORIAN
BEGIN:VEVENT
UID:132456-34365
SUMMARY:Weekly meeting
DTSTART:20120101T120000
DURATION:PT1H
RRULE:FREQ=WEEKLY
END:VEVENT
END:VCALENDAR
"#);
        });

        server.mock(|when, then| {
            when.path("/calendars/johndoe/tasks/")
                .header("Depth", "1")
                .body(r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:resourcetype />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            <c:comp-filter name="VTODO" />
        </c:comp-filter>
    </c:filter>
</c:calendar-query>
"#);

            then.status(207)
                .body(r#"
<d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:response>
        <d:href>/calendars/johndoe/tasks/132456762153245.ics</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype/>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>
"#);
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/calendars/johndoe/tasks/132456762153245.ics");

            then.status(200)
                .body(r#"
BEGIN:VCALENDAR
VERSION:2.0
CALSCALE:GREGORIAN
BEGIN:VTODO
UID:132456762153245
SUMMARY:Do the dishes
DUE:20121028T115600Z
END:VTODO
END:VCALENDAR
"#);
        });

        server.mock(|when, then| {
            when.path("/calendars/johndoe/home/")
                .header("Depth", "1")
                .body(r#"
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:prop>
        <d:resourcetype />
    </d:prop>
    <c:filter>
        <c:comp-filter name="VCALENDAR">
            <c:comp-filter name="VEVENT">
                <c:time-range start="20231028T000000Z" end="+infinity"/>
            </c:comp-filter>
        </c:comp-filter>
    </c:filter>
</c:calendar-query>"#
                );

            then.status(207)
                .body(r#"
<d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
    <d:response>
        <d:href>/calendars/johndoe/home/132456-34365.ics</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype/>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>
"#);
        });

        server
    }
}
