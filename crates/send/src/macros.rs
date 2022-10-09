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
            #[allow(clippy::too_many_arguments)]
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
macro_rules! packet_write_fn {
    (
        $($spec_name:ty => $fn_name:ident {
            $(mod $mod_name:path;)+
        }$(,)?)*
    ) => {
        $(
        pub fn $fn_name<B>(
            packet: $spec_name,
            buffer: &mut B,
            protocol: i32,
        ) -> ::std::result::Result<bool, ::falcon_packet_core::WriteError>
        where
            B: ::falcon_packet_core::special::PacketPrepare,
        {
            let mut packet = Some(packet);
            $(
            if $mod_name(&mut packet, buffer, protocol)? {
                return Ok(true);
            }
            )+
            Ok(false)
        }
        )*
    }
}
