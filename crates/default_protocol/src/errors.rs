error_chain! {
    links {
        Protocol(::falcon_protocol::errors::Error, ::falcon_protocol::errors::ErrorKind);
    }
}