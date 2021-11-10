use error_chain::error_chain;

error_chain! {
    foreign_links {
        Utf8(::std::string::FromUtf8Error);
    }
    errors {
        /// The PacketBuffer has no more bytes left.
        NoMoreBytes {
            display("PacketBuffer reached EOF!")
        }
        /// The received VarI32 is longer than 5 bytes.
        VarI32TooLong {
            display("VarI32 was longer than 5 bytes!")
        }
        /// The received VarI64 is longer than 10 bytes.
        VarI64TooLong {
            display("VarI64 was longer than 10 bytes!")
        }
        /// The received String data was not valid UTF-8.
        BadString {
            display("Invalid String!!")
        }
        /// The received String was empty.
        StringSizeZero {
            display("String size was 0!")
        }
        /// The received String was longer than the packet field allows for.
        StringTooLong {
            display("String was longer than expected!")
        }

        /// Plugin errors
        LibraryLoadingError(message: String) {
            display("Couldn't load library '{}'", message)
        }
    }
}
