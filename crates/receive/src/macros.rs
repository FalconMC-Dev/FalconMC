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

        pub fn falcon_process_packet<B>(packet_id: i32, buffer: &mut B, connection: &mut ::falcon_logic::connection::FalconConnection) -> ::anyhow::Result<bool>
        where
            B: ::bytes::Buf,
        {
            $(if $mod_name_rest::falcon_process_packet(packet_id, buffer, connection)? {
                return Ok(true);
            })*
            let connection_state = connection.state().connection_state;
            match connection_state {
                $(::falcon_core::network::ConnectionState::Handshake => {
                    $(if $mod_name_handshake::falcon_process_packet(packet_id, buffer, connection)? {
                        return Ok(true);
                    })*
                    Ok(false)
                },)?
                $(::falcon_core::network::ConnectionState::Status => {
                    $(if $mod_name_status::falcon_process_packet(packet_id, buffer, connection)? {
                        return Ok(true);
                    })*
                    Ok(false)
                },)?
                $(::falcon_core::network::ConnectionState::Login => {
                    $(if $mod_name_login::falcon_process_packet(packet_id, buffer, connection)? {
                        return Ok(true);
                    })*
                    Ok(false)
                },)?
                $(::falcon_core::network::ConnectionState::Play => {
                    $(if $mod_name_play::falcon_process_packet(packet_id, buffer, connection)? {
                        return Ok(true);
                    })*
                    Ok(false)
                },)?
                _ => Ok(false)
            }
        }
    }
}
