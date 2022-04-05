use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::character::complete::char;
use nom::combinator::map;
use nom::error::Error;
use nom::sequence::separated_pair;

#[derive(Debug, PartialEq)]
pub struct Identifier {
    namespace: Cow<'static, str>,
    location: Cow<'static, str>,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.location)
    }
}

impl Identifier {
    pub fn from_arbitrary(namespace: &str, location: &str) -> Self {
        let namespace = if namespace == "minecraft" {
            "minecraft".into()
        } else {
            Cow::from(namespace.to_string())
        };
        Identifier {
            namespace,
            location: Cow::from(location.to_string()),
        }
    }

    pub fn location_arbitrary(location: &str) -> Self {
        Identifier {
            namespace: "minecraft".into(),
            location: Cow::from(location.to_string()),
        }
    }

    pub fn from_static(namespace: &'static str, location: &'static str) -> Self {
        Identifier {
            namespace: namespace.into(),
            location: location.into(),
        }
    }

    pub fn location_static(location: &'static str) -> Self {
        Identifier {
            namespace: "minecraft".into(),
            location: location.into(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn parse_static(input: &'static str) -> Result<Self, usize> {
        let namespace_domain = take_while::<_, _, Error<&'static str>>(|i| "0123456789abcdefghijklmnopqrstuvwxyz-_.".contains(i));
        let location_domain = take_while::<_, _, Error<&'static str>>(|i| "0123456789abcdefghijklmnopqrstuvwxyz-_./".contains(i));
        let namespace_location = separated_pair(namespace_domain, char(':'), location_domain);
        let location_only = map(
            take_while::<_, _, Error<&'static str>>(|i| "0123456789abcdefghijklmnopqrstuvwxyz-_./".contains(i)),
            |o: &'static str| ("minecraft", o)
        );
        let (input, (namespace, location)) = alt((namespace_location, location_only))(input).unwrap();
        if input != "" {
            Err(location.len())
        } else {
            Ok(Identifier {
                namespace: namespace.into(),
                location: location.into(),
            })
        }
    }
}

impl<'a> TryFrom<&'a str> for Identifier {
    type Error = usize;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        let namespace_domain = take_while::<_, _, Error<&'a str>>(|i| "0123456789abcdefghijklmnopqrstuvwxyz-_.".contains(i));
        let location_domain = take_while::<_, _, Error<&'a str>>(|i| "0123456789abcdefghijklmnopqrstuvwxyz-_./".contains(i));
        let namespace_location = map(
            separated_pair(namespace_domain, char(':'), location_domain),
            |(namespace, location)| {
                (if namespace == "minecraft" {
                    "minecraft".into()
                } else {
                    Cow::from(namespace.to_string())
                }, Cow::from(location.to_string()))
            }
        );
        let location_only = map(
            take_while::<_, _, Error<&'a str>>(|i| "0123456789abcdefghijklmnopqrstuvwxyz-_./".contains(i)),
            |o: &'a str| ("minecraft".into(), Cow::from(o.to_string()))
        );
        let (input, (namespace, location)) = alt((namespace_location, location_only))(input).unwrap();
        if input != "" {
            Err(location.len())
        } else {
            Ok(Identifier {
                namespace,
                location,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use super::Identifier;

    #[test]
    fn it_works() {
        let test1 = "minecraft:test";
        let test2 = "minecraft:test/test/123.done";
        let test3 = Identifier {
            namespace: Cow::Borrowed("minecraft"),
            location: Cow::Borrowed("test"),
        };
        let test4 = Identifier {
            namespace: Cow::Borrowed("minecraft"),
            location: Cow::Owned(String::from("test")),
        };
        let test5 = "falcon:test/test/123.done";

        assert_eq!(format!("{}", Identifier::from_static("minecraft", "test")), test1);
        assert_eq!(format!("{}", Identifier::from_arbitrary("minecraft", "test")), test1);
        assert_eq!(format!("{}", Identifier::location_arbitrary("test")), test1);
        assert_eq!(format!("{}", Identifier::location_static("test")), test1);

        assert_eq!(format!("{}", Identifier::from_static("minecraft", "test/test/123.done")), test2);
        assert_eq!(format!("{}", Identifier::from_arbitrary("minecraft", "test/test/123.done")), test2);
        assert_eq!(format!("{}", Identifier::location_arbitrary("test/test/123.done")), test2);
        assert_eq!(format!("{}", Identifier::location_static("test/test/123.done")), test2);

        assert_eq!(format!("{}", Identifier::from_static("falcon", "test/test/123.done")), test5);
        assert_eq!(format!("{}", Identifier::from_arbitrary("falcon", "test/test/123.done")), test5);

        assert_eq!(Identifier::from_static("minecraft", "test"), test3);
        assert_eq!(Identifier::from_arbitrary("minecraft", "test"), test4);
        assert_eq!(Identifier::location_static("test"), test3);
        assert_eq!(Identifier::location_arbitrary("test"), test4);

        assert_eq!(Identifier::parse_static("minecraft:test").unwrap(), test3);
        assert_eq!(Identifier::parse_static("minecraft:te*st"), Err(2));
        assert_eq!(Identifier::try_from("minecraft:test").unwrap(), test4);
        assert_eq!(Identifier::try_from("mine*craft:test"), Err(4));
    }
}