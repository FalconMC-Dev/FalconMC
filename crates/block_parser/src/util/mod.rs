use proc_macro2::TokenStream;
use quote::quote;
use crate::EnumPropertyBase;
use crate::util::properties::EnumProperty;

pub mod data;
pub mod properties;
pub mod raw;
pub mod convert;
pub mod processing;

pub fn create_default_property_base() -> EnumPropertyBase {
    EnumPropertyBase::new(vec![
        EnumProperty::new("PortalAxis", vec!["x", "z"]),
        EnumProperty::new("HorizontalFacing", vec!["north", "south", "west", "east"]),
        EnumProperty::new(
            "HopperFacing",
            vec!["down", "north", "south", "west", "east"],
        ),
        EnumProperty::new(
            "AllFacing",
            vec!["north", "east", "south", "west", "up", "down"],
        ),
        EnumProperty::new("DoubleBlockHalf", vec!["upper", "lower"]),
        EnumProperty::new("SingleBlockHalf", vec!["top", "bottom"]),
        EnumProperty::new("ComparatorMode", vec!["compare", "subtract"]),
        EnumProperty::new("StructureBlockMode", vec!["save", "load", "corner", "data"]),
        EnumProperty::new(
            "StraightRailShape",
            vec![
                "north_south",
                "east_west",
                "ascending_east",
                "ascending_west",
                "ascending_north",
                "ascending_south",
            ],
        ),
        EnumProperty::new(
            "RailShape",
            vec![
                "north_south",
                "east_west",
                "ascending_east",
                "ascending_west",
                "ascending_north",
                "ascending_south",
                "south_east",
                "south_west",
                "north_west",
                "north_east",
            ],
        ),
        EnumProperty::new(
            "StairShape",
            vec![
                "straight",
                "inner_left",
                "inner_right",
                "outer_left",
                "outer_right",
            ],
        ),
        EnumProperty::new("ChestType", vec!["single", "left", "right"]),
        EnumProperty::new("PistonType", vec!["normal", "sticky"]),
        EnumProperty::new("RedstoneType", vec!["up", "side", "none"]),
        EnumProperty::new("SlabType", vec!["top", "bottom", "double"]),
        EnumProperty::new("WallType", vec!["none", "low", "tall"])])
}

pub fn generate_block_parse_error() -> TokenStream {
    quote!(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum ParseBlockError {
            UnknownBlock,
            UnknownProperty,
            InvalidProperty,
            InvalidToken,
        }

        impl ::std::error::Error for ParseBlockError {}

        impl ::std::fmt::Display for ParseBlockError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl From<std::str::ParseBoolError> for ParseBlockError {
            fn from(_: std::str::ParseBoolError) -> Self {
                ParseBlockError::InvalidProperty
            }
        }

        impl From<std::num::ParseIntError> for ParseBlockError {
            fn from(_: std::num::ParseIntError) -> Self {
                ParseBlockError::InvalidProperty
            }
        }
    )
}

pub fn avoid_type(mut input: String) -> String {
    if input == "type" {
        input.push('d');
    }
    input
}