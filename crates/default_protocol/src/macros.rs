#[macro_export]
macro_rules! implement_packet_handler_enum {
    ($name:ident, $( $variant:tt ),+) => {
        impl ::falcon_core::network::packet::PacketHandler for $name {
            fn handle_packet(self, connection: &mut dyn ::falcon_core::network::connection::MinecraftConnection) -> ::falcon_core::network::packet::PacketHandlerResult {
                match self {
                    $(
                        $name::$variant(inner) => inner.handle_packet(connection)
                    ),+
                }
            }

            fn get_name(&self) -> &'static str {
                match self {
                    $(
                        $name::$variant(inner) => inner.get_name()
                    ),+
                }
            }
        }
    }
}

#[macro_export]
macro_rules! packet_modules {
    (
    $(type Handshake => {
        $($visi_handshake:vis mod $mod_name_handshake:ident;)*
    })?
    $(,)?
    $(type Status => {
        $($visi_status:vis mod $mod_name_status:ident;)*
    })?
    $(,)?
    $(type Login => {
        $($visi_login:vis mod $mod_name_login:ident;)*
    })?
    $(,)?
    $(type Play => {
        $($visi_play:vis mod $mod_name_play:ident;)*
    })?
    $(,)?
    ) => {
        $($($visi_handshake mod $mod_name_handshake;)*)?
        $($($visi_status mod $mod_name_status;)*)?
        $($($visi_login mod $mod_name_login;)*)?
        $($($visi_play mod $mod_name_play;)*)?

        pub fn falcon_process_packet<R, C>(packet_id: i32, buffer: &mut R, connection: &mut C) -> Result<Option<()>, crate::error::DefaultProtocolError>
        where
            R: ::falcon_core::network::buffer::PacketBufferRead,
            C: ::falcon_core::network::connection::MinecraftConnection,
        {
            let connection_state = connection.handler_state().connection_state();
            match connection_state {
                $(::falcon_core::network::ConnectionState::Handshake => {
                    $(match $mod_name_handshake::falcon_process_packet(packet_id, buffer, connection)? {
                        Some(_) => return Ok(Some(())),
                        None => {}
                    })*
                    Ok(None)
                },)?
                $(::falcon_core::network::ConnectionState::Status => {
                    $(match $mod_name_status::falcon_process_packet(packet_id, buffer, connection)? {
                        Some(_) => return Ok(Some(())),
                        None => {}
                    })*
                    Ok(None)
                },)?
                $(::falcon_core::network::ConnectionState::Login => {
                    $(match $mod_name_login::falcon_process_packet(packet_id, buffer, connection)? {
                        Some(_) => return Ok(Some(())),
                        None => {}
                    })*
                    Ok(None)
                },)?
                $(::falcon_core::network::ConnectionState::Play => {
                    $(match $mod_name_play::falcon_process_packet(packet_id, buffer, connection)? {
                        Some(_) => return Ok(Some(())),
                        None => {}
                    })*
                    Ok(None)
                },)?
                _ => Ok(None)
            }
        }
    }
}
