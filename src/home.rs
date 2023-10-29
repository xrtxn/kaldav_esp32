use crate::Children;
use crate::Requestable;
use std::collections::HashMap;

#[derive(Clone, Debug, crate::Object)]
pub struct Home {
    url: String,
    auth: Option<crate::Authorization>,
}

impl Home {
    pub fn calendars(&self) -> crate::Result<HashMap<String, crate::Calendar>> {
        let response = self.propfind(&self.url, r#"
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
     <d:resourcetype />
     <d:displayname />
     <cs:getctag />
     <c:supported-calendar-component-set />
  </d:prop>
</d:propfind>
"#)?;

        Ok(self.to_map(
            &response,
            "//d:response//d:displayname/text()",
            "//d:displayname [text() = '{}']/../../../d:href/text()",
        ))
    }
}
