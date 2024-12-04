use crate::Requestable;

pub struct Iterator {
    objects: Vec<Object>,
    current: usize,
}

impl Iterator {
    pub(crate) fn from(objects: Vec<Object>) -> Self {
        Self {
            objects,
            current: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    fn get(&self, index: usize) -> crate::Result<ikal::VCalendar> {
        let object = &self.objects[index];
        let contents = object.get(object.url.clone())?;
        ikal::VCalendar::try_from(contents).map_err(crate::Error::from)
    }
}

impl std::iter::Iterator for Iterator {
    type Item = ikal::VCalendar;

    fn next(&mut self) -> Option<Self::Item> {
        self.objects.get(self.current)?;

        let component = self.get(self.current).unwrap();
        self.current += 1;

        Some(component)
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    url: String,
    auth: Option<crate::Authorization>,
}

impl crate::Children for Object {
    fn new<S>(url: S, _: &std::collections::BTreeMap<String, String>) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            auth: None,
        }
    }
}

impl crate::Xmlable for Object {
    fn url(&self) -> &str {
        &self.url
    }
}

impl crate::Requestable for Object {
    fn auth(&self) -> Option<crate::Authorization> {
        self.auth.clone()
    }

    fn set_auth(&mut self, auth: Option<crate::Authorization>) {
        self.auth = auth;
    }
}