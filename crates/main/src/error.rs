#![macro_use]

/// Prints error chain.
macro_rules! print_error {
    ($err:expr) => {{
        ::tracing::error!("ERROR: {}", $err);
        $err.chain()
            .skip(1)
            .for_each(|cause| ::tracing::error!("because: {}", cause));
    }};
}
