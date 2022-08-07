use syn::Error;
use syn::LitStr;

#[derive(Debug)]
pub struct SendFnName {
    name: Option<LitStr>,
    has_errored: bool,
}

impl SendFnName {
    pub fn new() -> Self {
        Self {
            name: None,
            has_errored: false,
        }
    }

    pub fn set_name(&mut self, name: LitStr) -> syn::Result<()> {
        if let Some(n) = &self.name {
            let mut error = Error::new(name.span(), "duplicate function name");
            if !self.has_errored {
                error.combine(Error::new(n.span(), "duplicate function name"));
                self.has_errored = true;
            }
            Err(error)
        } else {
            self.name = Some(name);
            Ok(())
        }
    }

    pub fn name(self) -> Option<LitStr> {
        self.name
    }
}
