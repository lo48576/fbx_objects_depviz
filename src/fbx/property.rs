//! Contians properties common to the FBX objects.

use std::io::Read;

use fbxcel::pull_parser::v7400::{attribute::loaders::DirectLoader, Attributes as Attributes7400};

#[derive(Debug, Clone)]
pub struct ObjectProperties {
    pub uid: i64,
    pub name: String,
    pub class: String,
    pub subclass: String,
}

impl ObjectProperties {
    pub fn from_attrs7400<R: Read>(attrs: Attributes7400<'_, R>) -> Option<Self> {
        let mut attrs = attrs.into_iter(std::iter::repeat(DirectLoader));
        let uid = attrs.next()?.unwrap().get_i64()?;
        let (name, class) = attrs
            .next()?
            .unwrap()
            .get_string()
            .and_then(separate_name_class)
            .map(|(n, c)| (n.to_owned(), c.to_owned()))?;
        let subclass = attrs.next()?.unwrap().get_string()?.to_owned();

        Some(Self {
            uid,
            name,
            class,
            subclass,
        })
    }
}

/// Returns `Option<(name: String, class: String)>`
fn separate_name_class(name_class: &str) -> Option<(&str, &str)> {
    #[allow(clippy::manual_map)]
    if let Some(sep_pos) = name_class.find("\u{0}\u{1}") {
        // String is "name\u{0}\u{1}class" format.
        Some((&name_class[0..sep_pos], &name_class[sep_pos + 2..]))
    } else if let Some(sep_pos) = name_class.find("::") {
        // String is "class::name" format.
        Some((&name_class[sep_pos + 2..], &name_class[0..sep_pos]))
    } else {
        None
    }
}
