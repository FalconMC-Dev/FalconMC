#![macro_use]

use error_chain::error_chain;

error_chain! {
    links {
        Core(::falcon_core::errors::Error, ::falcon_core::errors::ErrorKind);
    }
}

macro_rules! print_error {
    ($e:ident) => {{
        error!("error: {}", $e);
        for e in $e.iter().skip(1) {
            error!("caused by: {}", e);
        }

        if let Some(backtrace) = $e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }
    }};
}
