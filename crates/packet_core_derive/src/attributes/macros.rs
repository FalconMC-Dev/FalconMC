macro_rules! impl_parse {
    ($first_v:ident = ($first_k:ident as $first_i:expr) $(,$($other_v:ident = ($other_k:ident as $other_i:expr)),+)?$(,)?) => {
        impl ::syn::parse::Parse for PacketAttribute {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                if input.peek($first_i) {
                    Ok(Self::$first_v(input.parse::<$first_k>()?))
                }
                $($(else if input.peek($other_i) {
                    Ok(Self::$other_v(input.parse::<$other_k>()?))
                })+)?
                else {
                    Err(::syn::Error::new(input.span(), "Invalid attribute(s)"))
                }
            }
        }
    }
    // ($variant:ident = ($kind:ident as $name:expr) $(,$tokens:tt)?$(,)?) => {
    //     impl_parse! { @first $variant = ($kind as $name) $(@rest $tokens)? }
    // };
    // (@first $first_v:ident = ($first_k:ident as $first_i:expr) $(@rest $($other_v:ident = ($other_k:ident as $other_i:expr)),+)?) => {
    //     impl syn::parse::Parse for PacketAttribute {
    //         fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    //             if input.peek($first_i) {
    //                 Ok(Self::$first_v(input.parse::<$first_k>()?))
    //             }
    //             $($(else if input.peek($other_i) {
    //                 Ok(Self::$other_v(input.parse::<$other_k>()?))
    //             })+)?
    //             else {
    //                 Err(::syn::Error::new(input.span(), "Invalid attribute(s)"))
    //             }
    //         }
    //     }
    // };
}
