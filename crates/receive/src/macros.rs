#[macro_export]
macro_rules! packet_modules {
    (
    $(extern $visi_rest:vis mod $mod_name_rest:ident;)*
    $(type Handshake => {
        $($visi_handshake:vis mod $mod_name_handshake:ident;)*
    })?
    $(;)?
    $(type Status => {
        $($visi_status:vis mod $mod_name_status:ident;)*
    })?
    $(;)?
    $(type Login => {
        $($visi_login:vis mod $mod_name_login:ident;)*
    })?
    $(;)?
    $(type Play => {
        $($visi_play:vis mod $mod_name_play:ident;)*
    })?
    $(;)?
    ) => {
        $($visi_rest mod $mod_name_rest;)*
        $($($visi_handshake mod $mod_name_handshake;)*)?
        $($($visi_status mod $mod_name_status;)*)?
        $($($visi_login mod $mod_name_login;)*)?
        $($($visi_play mod $mod_name_play;)*)?

        pub fn falcon_process_packet<R>(packet_id: i32, buffer: &mut R, connection: &mut ::falcon_logic::connection::FalconConnection) -> ::core::result::Result<::core::option::Option<()>, ::falcon_core::error::FalconCoreError>
        where
            R: ::falcon_core::network::buffer::PacketBufferRead,
        {
            $(if $mod_name_rest::falcon_process_packet(packet_id, buffer, connection)?.is_some() {
                return Ok(Some(()));
            })*
            use ::falcon_core::network::connection::ConnectionLogic;
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
