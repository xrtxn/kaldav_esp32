mod calendar;
mod client;
mod event;
mod home;
mod principal;
mod result;

use calendar::*;
use event::*;
use home::*;
use principal::*;

pub use client::*;
pub use result::*;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Authorization {
    pub username: String,
    pub password: Option<String>,
}

pub trait Requestable {
    fn get_auth(&self) -> Option<Authorization>;
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
        let mut headers = HashMap::new();

        headers.insert("Depth", "1");

        self.request("REPORT", href, Some(body), Some(headers))
    }

    fn request<S>(
        &self,
        method: &str,
        href: S,
        body: Option<&str>,
        headers: Option<HashMap<&'static str, &'static str>>,
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

        if let Some(auth) = self.get_auth() {
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
    fn get_url(&self) -> String;

    fn get_xml(xml: &str, xpath: &str) -> Vec<String> {
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
        let url = url::Url::parse(self.get_url().as_str()).unwrap();

        format!("{}://{}/{}", url.scheme(), url.host_str().unwrap(), href)
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
        Self::get_xml(response, xpath)
            .iter()
            .map(|x| {
                let mut element = C::new(self.append_host(x.clone()));

                element.set_auth(self.get_auth());

                element
            })
            .collect()
    }

    fn to_map<C>(&self, response: &str, key_xpath: &str, value_xpath: &str) -> HashMap<String, C>
    where
        C: Children + Requestable,
    {
        let mut map = HashMap::new();
        let keys = Self::get_xml(response, key_xpath);

        for key in keys.iter().skip(1) {
            let xpath = value_xpath.replace("{}", key.as_str());
            let values = Self::get_xml(response, xpath.as_str());

            let mut element = C::new(self.append_host(values[0].clone()));
            element.set_auth(self.get_auth());

            map.insert(key.to_string(), element);
        }

        map
    }
}
