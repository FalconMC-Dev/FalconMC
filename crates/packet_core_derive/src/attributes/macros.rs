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
}

macro_rules! none_except {
    ($($except:pat_param)|+, $iter:ident, $this:expr) => {
        {
            let mut error = ErrorCatcher::new();
            $iter.for_each(|a| {
                match a {
                    $($except)|+ => {}
                    a => error.add_error(Error::new(
                        a.span(),
                        format!("Incompatible with {}", $this),
                    ))
                }
            });
            error
        }
    }
}

macro_rules! all_except {
    ($($except:pat_param)|+, $iter:ident, $this:expr) => {
        {
            let mut error = ErrorCatcher::new();
            $iter.for_each(|a| {
                match a {
                    $($except)|+ => error.add_error(Error::new(
                        a.span(),
                        format!("Incompatible with {}", $this),
                    )),
                    _ => {}
                }
            });
            error
        }
    }
}

// macro_rules! none {
//     ($iter:ident, $this:expr) => {
//         {
//             let mut error = ErrorCatcher::new();
//             $iter.for_each(|a| {
//                 error.add_error(Error::new(
//                     a.span(),
//                     format!("Incompatible with {}", $this),
//                 ))
//             });
//             error.emit()
//         }
//     }
// }
