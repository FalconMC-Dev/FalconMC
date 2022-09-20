use std::collections::HashSet;

use falcon_proc_util::ErrorCatcher;
use syn::{Error, Field, Ident};

use crate::attributes::PacketAttribute::{
    self, Array, Bytes, Convert, From, Into, Link, Nbt, String, VarI32, VarI64, Vec as PacketVec,
};

pub fn get_replaced(attributes: &[(&Field, Vec<PacketAttribute>)]) -> HashSet<Ident> {
    attributes
        .iter()
        .flat_map(|(_, attrs)| attrs.iter())
        .filter_map(|a| match a {
            PacketVec(data) => Some(data.target.clone()),
            Bytes(data) => data.target.clone(),
            Link(data) => Some(data.target.clone()),
            _ => None,
        })
        .collect()
}

pub fn is_outer(attribute: &PacketAttribute) -> bool {
    match attribute {
        String(_) => true,
        VarI32(_) => true,
        VarI64(_) => true,
        Bytes(_) => true,
        PacketVec(_) => true,
        Link(_) => true,
        Into(_) => false,
        From(_) => false,
        Convert(_) => false,
        Array(_) => true,
        Nbt(_) => true,
    }
}

pub fn validate(mut attributes: Vec<PacketAttribute>) -> syn::Result<Vec<PacketAttribute>> {
    let mut checked = Vec::with_capacity(attributes.len());
    let mut error = ErrorCatcher::new();

    for _ in 0..attributes.len() {
        let mut attribute = attributes.remove(0);
        error.extend_error(check(&mut attribute, attributes.iter_mut()));
        if is_outer(&attribute) && !attributes.is_empty() {
            error.add_error(Error::new(
                attribute.span(),
                "Ending attribute should be last in the list",
            ));
        }
        checked.push(attribute);
    }

    error.emit()?;
    Ok(checked)
}

pub fn check<'a, I>(current: &mut PacketAttribute, others: I) -> syn::Result<()>
where
    I: Iterator<Item = &'a mut PacketAttribute>,
{
    match current {
        String(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`string`").emit(),
        VarI32(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`var32`").emit(),
        VarI64(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`var64`").emit(),
        PacketVec(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`vec`").emit(),
        Link(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`link`").emit(),
        Into(_) => all_except!(Convert(_), others, "`into`").emit(),
        Convert(_) => all_except!(Into(_) | From(_), others, "`convert`").emit(),
        Array(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`array`").emit(),
        Bytes(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`bytes`").emit(),
        Nbt(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`nbt`").emit(),
        From(_) => Ok(()),
    }
}
