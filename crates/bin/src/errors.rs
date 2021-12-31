#![macro_use]

error_chain! {
    links {
        Core(::falcon_core::errors::Error, ::falcon_core::errors::ErrorKind);
        Protocol(::falcon_protocol::errors::Error, ::falcon_protocol::errors::ErrorKind);
    }
    errors {
        InvalidPacketLength {
            description("Incoming packet size was longer than 21 bits"),
            display("Incoming packet size was longer than 21 bits!"),
        }
    }
}

/// Prints error chain and backtrace.
macro_rules! print_error {
    ($err:expr) => {{
        let e: &$crate::errors::Error = $err;
        ::tracing::error!("error: {}", e);
        for e in e.iter().skip(1) {
            ::tracing::error!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            ::tracing::error!("backtrace: {:?}", backtrace);
        }
    }};
}

/// Avoid using this, this is a hack to get the `chain_err` effect on any error type.\
/// Returns a reference.
macro_rules! arbitrary_error {
    ($err:expr, $chained:expr) => {{
        let state =
            ::error_chain::State::new::<$crate::errors::Error>(::std::boxed::Box::new($err));
        &::error_chain::ChainedError::new($chained, state)
    }};
}
