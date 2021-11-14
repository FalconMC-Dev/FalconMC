error_chain! {
    links {
        Core(::falcon_core::errors::Error, ::falcon_core::errors::ErrorKind);
        Protocol(::falcon_protocol::errors::Error, ::falcon_protocol::errors::ErrorKind);
    }
}
