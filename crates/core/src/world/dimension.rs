use std::borrow::Cow;

pub struct Dimension {
    name: Cow<'static, str>,
    id: i32,
}

impl Dimension {
    pub fn new<S: Into<Cow<'static, str>>>(name: S, id: i32) -> Self {
        Dimension {
            name: name.into(),
            id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}
