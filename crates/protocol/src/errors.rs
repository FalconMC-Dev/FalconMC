#![macro_use]

error_chain! {
    links {
        Core(::falcon_core::errors::Error, ::falcon_core::errors::ErrorKind);
    }

    errors {
        LibraryLoadingError(name: ::std::ffi::OsString, t: String) {
            description("Cannot load library"),
            display("Could not load library {:?} due to {}", name, t)
        }
    }
}

/// Prints error chain and backtrace.
macro_rules! print_error {
    ($err:expr) => {{
        let e: &$crate::errors::Error = $err;
        ::log::error!("error: {}", e);
        for e in e.iter().skip(1) {
            ::log::error!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            ::log::error!("backtrace: {:?}", backtrace);
        }
    }};
}