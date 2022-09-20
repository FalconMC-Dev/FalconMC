#[falcon_send_derive::falcon_send]
mod inner {
    use crate::LoginSuccessSpec;
    use derive_from_ext::From;
    use falcon_packet_core::{PacketSize, PacketWrite};
    use uuid::Uuid;

    #[derive(PacketSize, PacketWrite, From)]
    #[from(LoginSuccessSpec)]
    #[falcon_packet(versions = {
        735, 736 = 0x02;
    }, name = "login_success")]
    pub struct LoginSuccessPacket {
        uuid: Uuid,
        #[falcon(string = 16)]
        username: String,
    }
}
