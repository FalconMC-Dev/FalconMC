use crate::packet_modules;

packet_modules! {
    type Handshake => {
        pub mod handshake;
    }
    type Status => {
        pub mod status;
    }
    type Login => {
        pub mod login;
    }
    type Play => {
        pub mod play;
    }
}
