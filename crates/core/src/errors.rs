use error_chain::error_chain;

error_chain! {
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

        InvalidSchematicVersion(version: i32) {
            display("Found version {} instead of 1 or 2", version)
        }
        WrongDataVersion(version: i32, required: i32) {
            display("Invalid data version received, should be {} instead of {}", version, required)
        }
        InvalidData {
            display("Invalid data was found")
        }
        MissingData {
            display("Could not find the right data describing the blocks")
        }
    }
}
