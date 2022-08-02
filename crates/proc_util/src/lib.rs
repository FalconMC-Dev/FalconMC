#[derive(Debug)]
pub struct ErrorCatcher {
    error: Option<syn::Error>,
}

impl ErrorCatcher {
    pub fn new() -> Self {
        Self {
            error: None,
        }
    }

    pub fn add_error(&mut self, error: syn::Error) {
        match self.error {
            Some(ref mut err) => err.combine(error),
            None => self.error = Some(error)
        }
    }

    pub fn extend_error(&mut self, error: Result<(), syn::Error>) {
        match error {
            Ok(value) => value,
            Err(error) => self.add_error(error),
        }
    }

    pub fn emit(self) -> syn::Result<()> {
        if let Some(err) = self.error {
            Err(err)
        } else {
            Ok(())
        }
    }
}
