#[macro_export]
macro_rules! implement_packet_handler_enum {
    ($name:ident, $( $variant:tt ),+) => {
        impl ::falcon_core::network::packet::PacketHandler for $name {
            fn handle_packet(self, connection: &mut dyn ::falcon_core::network::connection::MinecraftConnection) {
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