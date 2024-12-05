mod calendar;
mod client;
mod home;
mod object;
mod principal;
mod result;

pub use calendar::*;
pub use client::*;
pub use home::*;
pub use object::*;
pub use principal::*;
pub use result::*;

pub use ikal as ical;

use base64::Engine;
use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::utils::io;
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection, Method};
use esp_idf_svc::io::Write;
use kaldav_derive::*;
use std::collections::BTreeMap;
use std::iter::Iterator;

#[derive(Clone, Debug)]
pub struct Authorization {
    pub username: String,
    pub password: Option<String>,
}

pub trait Requestable {
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
        let href = href.into();
        println!("Requesting {} {}", method, href);

        let config = &HttpConfiguration {
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            use_global_ca_store: true,
            ..Default::default()
        };

        let mut converted_headers: Vec<(&str, &str)> = vec![];

        if let Some(headers) = headers {
            //todo check for no headers
            converted_headers = headers.iter().map(|(k, v)| (*k, *v)).collect();
        }

        let mut client = HttpClient::wrap(EspHttpConnection::new(&config).unwrap());

        let auth_header: String;
        if let Some(auth) = self.auth() {
            auth_header = format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(format!(
                    "{}:{}",
                    auth.username,
                    auth.password.unwrap()
                ))
            );
            converted_headers.push(("Authorization", auth_header.as_str()));
        }
        let body = body.unwrap();
        let binding = body.len().to_string();
        converted_headers.push(("Content-Length", binding.as_str()));

        println!("Headers: {:?}", converted_headers);

        let req_method = match method {
            "PROPFIND" => Method::Propfind,
            "REPORT" => Method::Report,
            "GET" => Method::Get,
            _ => panic!("Method not supported"),
        };

        let mut request = client.request(req_method, href.as_str(), &converted_headers)?;
        request.write_all(body.as_bytes())?;
        request.flush()?;

        let mut response = request.submit()?;

        let mut buf = [0u8; 8192];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        let text = std::str::from_utf8(&buf[0..bytes_read]).unwrap();

        if embedded_svc::http::status::OK.contains(&response.status()) {
            println!("Response: {}", text);
            Ok(text.to_string())
        } else {
            Err(Error::new(format!(
                "{method} {href}: {}",
                response.status()
            )))
        }
    }
}

pub trait Xmlable {
    fn url(&self) -> &str;

    fn xml(xml: &str, xpath: &str) -> Vec<String> {
        let package = sxd_document::parser::parse(xml).unwrap();
        let document = package.as_document();
        let root = document.root().children()[0];

        let mut context = sxd_xpath::Context::new();
        context.set_namespace("d", "DAV:");
        context.set_namespace("cal", "urn:ietf:params:xml:ns:caldav");
        context.set_namespace("x1", "http://apple.com/ns/ical/");

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

pub trait Children: Requestable + Xmlable {
    fn new<S>(url: S, params: &BTreeMap<String, String>) -> Self
    where
        S: Into<String>;

    fn one<C>(&self, response: &str, xpath: &str) -> Option<C>
    where
        C: Children + Requestable,
    {
        let mut items = self.to_vec(response, xpath);

        if items.is_empty() {
            None
        } else {
            Some(items.remove(0))
        }
    }

    fn to_vec<C>(&self, response: &str, xpath: &str) -> Vec<C>
    where
        C: Children + Requestable,
    {
        Self::xml(response, xpath)
            .iter()
            .map(|x| {
                let mut element = C::new(self.append_host(x.clone()), &BTreeMap::new());

                element.set_auth(self.auth());

                element
            })
            .collect()
    }

    fn to_map<C>(
        &self,
        response: &str,
        key_xpath: &str,
        value_xpath: &str,
        params_xpath: Vec<(&str, &str)>,
    ) -> BTreeMap<String, C>
    where
        C: Children + Requestable,
    {
        let mut map = BTreeMap::new();
        let keys = Self::xml(response, key_xpath);

        for key in keys {
            let xpath = value_xpath.replace("{}", key.as_str());
            let values = Self::xml(response, xpath.as_str());

            let mut params = BTreeMap::new();
            for (param_name, param_xpath) in &params_xpath {
                let xpath = param_xpath.replace("{}", key.as_str());
                if let Some(param) = Self::xml(response, &xpath).first() {
                    params.insert(param_name.to_string(), param.clone());
                }
            }

            let mut element = C::new(self.append_host(values[0].clone()), &params);
            element.set_auth(self.auth());

            map.insert(key.to_string(), element);
        }

        map
    }
}

// #[cfg(test)]
// mod test {
//     pub(crate) fn server() -> httpmock::MockServer {
//         env_logger::try_init().ok();
//         let server = httpmock::MockServer::start();
//
//         server.mock(|when, then| {
//             when.path("/").body(
//                 r#"
// <d:propfind xmlns:d="DAV:">
//     <d:prop>
//         <d:current-user-principal />
//     </d:prop>
// </d:propfind>
// "#,
//             );
//             then.status(207).body(
//                 r#"
// <d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/">
//     <d:response>
//         <d:href>/</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:current-user-principal>
//                     <d:href>/principals/users/johndoe/</d:href>
//                 </d:current-user-principal>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
// </d:multistatus>"#,
//             );
//         });
//
//         server.mock(|when, then| {
//             when.path("/principals/users/johndoe/").body(
//                 r#"
// <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
//   <d:prop>
//      <d:displayname />
//      <c:calendar-home-set />
//   </d:prop>
// </d:propfind>
// "#,
//             );
//
//             then.status(207).body(
//                 r#"
// <d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:response>
//         <d:href>/principals/users/johndoe/</d:href>
//         <d:propstat>
//             <d:prop>
//                 <c:calendar-home-set>
//                     <d:href>/calendars/johndoe/</d:href>
//                 </c:calendar-home-set>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
// </d:multistatus>
// "#,
//             );
//         });
//
//         server.mock(|when, then| {
//             when.path("/calendars/johndoe/")
//                 .body(r#"
// <d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:x1="http://apple.com/ns/ical/">
//   <d:prop>
//      <d:resourcetype />
//      <d:displayname />
//      <cs:getctag />
//      <c:supported-calendar-component-set />
//      <x1:calendar-color />
//   </d:prop>
// </d:propfind>
// "#);
//
//             then.status(207)
//                 .body(r#"
// <d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:response>
//         <d:href>/calendars/johndoe/</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:resourcetype>
//                     <d:collection/>
//                 </d:resourcetype>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
//     <d:response>
//         <d:href>/calendars/johndoe/home/</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:resourcetype>
//                     <d:collection/>
//                     <c:calendar/>
//                 </d:resourcetype>
//                 <d:displayname>Home calendar</d:displayname>
//                 <cs:getctag>3145</cs:getctag>
//                 <c:supported-calendar-component-set>
//                     <c:comp name="VEVENT" />
//                 </c:supported-calendar-component-set>
//                 <x1:calendar-color xmlns:x1="http://apple.com/ns/ical/">#ffd4a5</x1:calendar-color>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
//     <d:response>
//         <d:href>/calendars/johndoe/tasks/</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:resourcetype>
//                     <d:collection/>
//                     <c:calendar/>
//                 </d:resourcetype>
//                 <d:displayname>My TODO list</d:displayname>
//                 <cs:getctag>3345</cs:getctag>
//                 <c:supported-calendar-component-set>
//                     <c:comp name="VTODO" />
//                 </c:supported-calendar-component-set>
//                 <x1:calendar-color xmlns:x1="http://apple.com/ns/ical/">#ad0083</x1:calendar-color>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
// </d:multistatus>
// "#);
//         });
//
//         server.mock(|when, then| {
//             when.path("/calendars/johndoe/home/")
//                 .header("Depth", "1")
//                 .body(r#"
// <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:prop>
//         <d:resourcetype />
//     </d:prop>
//     <c:filter>
//         <c:comp-filter name="VCALENDAR">
//             <c:comp-filter name="VEVENT" />
//         </c:comp-filter>
//     </c:filter>
// </c:calendar-query>
// "#);
//
//             then.status(207)
//                 .body(r#"
// <d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:response>
//         <d:href>/calendars/johndoe/home/132456-34365.ics</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:resourcetype/>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
// </d:multistatus>
// "#);
//         });
//
//         server.mock(|when, then| {
//             when.method(httpmock::Method::GET)
//                 .path("/calendars/johndoe/home/132456-34365.ics");
//
//             then.status(200).body(
//                 "BEGIN:VCALENDAR\r
// VERSION:2.0\r
// CALSCALE:GREGORIAN\r
// PRODID:kaldav\r
// BEGIN:VEVENT\r
// DTSTAMP:20120101T120000\r
// UID:132456-34365\r
// SUMMARY:Weekly meeting\r
// DTSTART:20120101T120000\r
// DURATION:PT1H\r
// RRULE:FREQ=WEEKLY\r
// END:VEVENT\r
// END:VCALENDAR\r
// ",
//             );
//         });
//
//         server.mock(|when, then| {
//             when.path("/calendars/johndoe/tasks/")
//                 .header("Depth", "1")
//                 .body(r#"
// <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:prop>
//         <d:resourcetype />
//     </d:prop>
//     <c:filter>
//         <c:comp-filter name="VCALENDAR">
//             <c:comp-filter name="VTODO" />
//         </c:comp-filter>
//     </c:filter>
// </c:calendar-query>
// "#);
//
//             then.status(207)
//                 .body(r#"
// <d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:response>
//         <d:href>/calendars/johndoe/tasks/132456762153245.ics</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:resourcetype/>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
// </d:multistatus>
// "#);
//         });
//
//         server.mock(|when, then| {
//             when.method(httpmock::Method::GET)
//                 .path("/calendars/johndoe/tasks/132456762153245.ics");
//
//             then.status(200).body(
//                 "BEGIN:VCALENDAR\r
// VERSION:2.0\r
// PRODID:kaldav\r
// CALSCALE:GREGORIAN\r
// BEGIN:VTODO\r
// DTSTAMP:20120101T120000\r
// UID:132456762153245\r
// SUMMARY:Do the dishes\r
// DUE:20121028T115600Z\r
// END:VTODO\r
// END:VCALENDAR\r
// ",
//             );
//         });
//
//         server.mock(|when, then| {
//             when.path("/calendars/johndoe/home/")
//                 .header("Depth", "1")
//                 .body(r#"
// <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:prop>
//         <d:resourcetype />
//     </d:prop>
//     <c:filter>
//         <c:comp-filter name="VCALENDAR">
//             <c:comp-filter name="VEVENT">
//                 <c:time-range start="20231028T000000Z" end="+infinity"/>
//             </c:comp-filter>
//         </c:comp-filter>
//     </c:filter>
// </c:calendar-query>"#
//                 );
//
//             then.status(207)
//                 .body(r#"
// <d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
//     <d:response>
//         <d:href>/calendars/johndoe/home/132456-34365.ics</d:href>
//         <d:propstat>
//             <d:prop>
//                 <d:resourcetype/>
//             </d:prop>
//             <d:status>HTTP/1.1 200 OK</d:status>
//         </d:propstat>
//     </d:response>
// </d:multistatus>
// "#);
//         });
//
//         server
//     }
// }
