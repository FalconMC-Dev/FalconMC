use falcon_proc_util::ErrorCatcher;
use syn::Error;

use crate::attributes::PacketAttribute::{
    self, Array, AsRef, Bytes, Convert, From, Into, String, VarI32, VarI64, Vec as PacketVec,
};

pub fn is_outer(attribute: &PacketAttribute) -> bool {
    match attribute {
        String(_) => true,
        VarI32(_) => true,
        VarI64(_) => true,
        Bytes(_) => true,
        PacketVec(_) => true,
        Into(_) => false,
        From(_) => false,
        Convert(_) => false,
        Array(_) => true,
        AsRef(_) => true,
    }
}

pub fn validate(mut attributes: Vec<PacketAttribute>) -> syn::Result<Vec<PacketAttribute>> {
    let mut checked = Vec::with_capacity(attributes.len());
    let mut error = ErrorCatcher::new();

    for _ in 0..attributes.len() {
        let mut attribute = attributes.remove(0);
        error.extend_error(check(&mut attribute, attributes.iter_mut()));
        if !matches!(attribute, Bytes(_))
            && is_outer(&attribute)
            && attributes.iter().any(|e| !matches!(e, Bytes(_)))
        {
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
        Into(_) => all_except!(Convert(_), others, "`into`").emit(),
        Convert(_) => all_except!(Into(_) | From(_), others, "`convert`").emit(),
        Array(_) => none_except!(Into(_) | From(_) | Convert(_), others, "`array`").emit(),
        AsRef(data) => {
            let mut error = ErrorCatcher::new();
            others.for_each(|a| match a {
                Into(_) | From(_) | Convert(_) => {}
                Bytes(bytes) => data.target = bytes.target.clone(),
                a => error.add_error(Error::new(a.span(), "Incompatible with `asref`")),
            });
            error.emit()
        }
        Bytes(bytes) => {
            others.for_each(|a| {
                if let AsRef(data) = a {
                    data.target = bytes.target.clone();
                }
            });
            Ok(())
        }
        From(_) => Ok(()),
    }
}
