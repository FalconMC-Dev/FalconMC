use std::cell::RefCell;
use std::fmt::{Display, Formatter, Write};
use std::ops::Range;
use std::sync::Mutex;

use convert_case::{Case, Casing};
use once_cell::sync::Lazy;

static PROPERTIES: Lazy<Mutex<RefCell<Vec<EnumProperty>>>> = Lazy::new(|| {
    Mutex::new(RefCell::new(vec![
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
        EnumProperty::new("WallType", vec!["none", "low", "tall"]),
    ]))
});

pub fn display_enum_properties<W: Write>(writer: &mut W) {
    let list = PROPERTIES.lock().unwrap();
    list.borrow()
        .iter()
        .for_each(|x| write!(writer, "{}", x).unwrap());
}

#[derive(Clone, Debug, Eq)]
pub enum PropertyType {
    Bool,
    Int(Range<u8>),
    Enum((EnumProperty, Option<EnumProperty>)),
}

impl PartialEq for PropertyType {
    fn eq(&self, other: &Self) -> bool {
        let self_discriminant = std::mem::discriminant(self);
        let other_discriminant = std::mem::discriminant(other);
        if self_discriminant == other_discriminant {
            match (self, other) {
                (&PropertyType::Int(ref range1), &PropertyType::Int(ref range2)) => {
                    range1 == range2
                }
                (&PropertyType::Enum((ref target1, _)), &PropertyType::Enum((ref target2, _))) => {
                    target1 == target2
                }
                _ => true,
            }
        } else {
            false
        }
    }
}

impl Display for PropertyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::Bool => f.write_str("bool"),
            PropertyType::Int(_) => f.write_str("i32"),
            PropertyType::Enum((target, _real)) => f.write_fmt(format_args!("{}", target.name)),
        }
    }
}

impl PropertyType {
    pub fn from_raw(name: String, values: Vec<String>) -> Self {
        if values.first().unwrap() == "true" {
            return PropertyType::Bool;
        } else if let Ok(start) = values.first().unwrap().parse::<u8>() {
            return PropertyType::Int(start..(values.last().unwrap().parse::<u8>().unwrap() + 1));
        }

        let property = EnumProperty::new(name.to_case(Case::Pascal), values);
        let list = PROPERTIES.lock().unwrap();
        if list
            .borrow()
            .iter()
            .find(|x| x.fields.eq(&property.fields) || x.name.eq(&property.name))
            .is_none()
        {
            list.borrow_mut().push(property.clone());
            return PropertyType::Enum((property, None));
        }
        let x = PropertyType::Enum((
            list.borrow()
                .iter()
                .find(|x| x.fields.eq(&property.fields) || x.name.eq(&property.name))
                .unwrap()
                .clone(),
            None,
        ));
        x
    }

    pub fn find(name: String, values: Vec<String>) -> Option<Self> {
        if values.first().unwrap() == "true" {
            return Some(PropertyType::Bool);
        } else if let Ok(start) = values.first().unwrap().parse::<u8>() {
            return Some(PropertyType::Int(
                start..(values.last().unwrap().parse::<u8>().unwrap() + 1),
            ));
        }
        let property = EnumProperty::new(name.to_case(Case::Pascal), values);
        let list = PROPERTIES.lock().unwrap();
        let list = list.borrow();
        if let Some(prop) = match list
            .iter()
            .find(|x| x.fields.eq(&property.fields) || x.name.eq(&property.name))
        {
            Some(res) => {
                for (i, entry) in property.fields.iter().enumerate() {
                    if res.fields.get(i) != Some(entry) {
                        println!("Fields cannot be merged for {}! D:", property.name);
                        return None;
                    }
                }
                Some(res)
            }
            None => list
                .iter()
                .find(|x| property.fields.iter().all(|y| x.fields.contains(y))),
        } {
            return Some(PropertyType::Enum((prop.clone(), Some(property))));
        } else {
            println!("Could not find property {}", property.name);
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumProperty {
    name: String,
    pub fields: Vec<String>,
}

impl EnumProperty {
    /// Expects `PascalCase` for the name and `snake_case` for the fields
    pub fn new<T: ToString>(name: T, mut fields: Vec<T>) -> Self
    where
        String: From<T>,
    {
        EnumProperty {
            name: name.into(),
            fields: fields.drain(..).map(|val| val.into()).collect(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl Display for EnumProperty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]\npub enum {} {{\n",
            self.name
        )?;
        for field in &self.fields {
            write!(f, "    {},\n", field.to_case(Case::Pascal))?;
        }
        write!(f, "}}\n")
    }
}
