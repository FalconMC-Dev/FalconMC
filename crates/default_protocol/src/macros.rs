#[macro_export]
macro_rules! implement_packet_handler_enum {
    ($name:ident, $( $variant:tt ),+) => {
        impl ::falcon_core::network::packet::PacketHandler for $name {
            fn handle_packet(self, connection: &mut dyn ::falcon_core::network::connection::MinecraftConnection) -> ::falcon_core::network::packet::TaskScheduleResult {
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
macro_rules! define_spec {
    ($spec_name:ident $(=> $($arg:ident: $arg_ty:ty),*)? {
        $($default:ident: $default_ty:ty),*$(,)?
        $(;$(let $field:ident: $field_ty:ty = $init:expr),*$(,)?)?
        $(;{$($data:stmt)*})?
    }$(, $($trait:path),*)?) => {
        $($(#[derive($trait)])*)?
        pub struct $spec_name {
            $($(pub(crate) $field: $field_ty,)*)?
            $(pub(crate) $default: $default_ty),*
        }

        impl $spec_name {
            pub fn new($($($arg: $arg_ty,)*)? $($default: $default_ty),*) -> Self {
                $($($data)*)?
                $spec_name {
                    $($($field: $init,)*)?
                    $($default),*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! packet_send_fn {
    (
        $($spec_name:path => $fn_name:ident {
            $(mod $mod_name:path;)+
        }$(,)?)*
    ) => {
        $(
        pub fn $fn_name<C>(packet: $spec_name, connection: &mut C)
        where
            C: ::falcon_core::network::connection::MinecraftConnection + ?Sized,
        {
            let mut packet = Some(packet);
            $(
            if $mod_name(&mut packet, connection) {
                return;
            }
            )+
        }
        )*
    }
}

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

        pub fn falcon_process_packet<R, C>(packet_id: i32, buffer: &mut R, connection: &mut C) -> Result<Option<()>, crate::error::DefaultProtocolError>
        where
            R: ::falcon_core::network::buffer::PacketBufferRead,
            C: ::falcon_core::network::connection::MinecraftConnection,
        {
            $(if $mod_name_rest::falcon_process_packet(packet_id, buffer, connection)?.is_some() {
                return Ok(Some(()));
            })*
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
