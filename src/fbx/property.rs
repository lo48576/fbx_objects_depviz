//! Contians properties common to the FBX objects.

use fbx_direct::common::OwnedProperty as NodeProperty;

#[derive(Debug, Clone)]
pub struct ObjectProperties {
    pub uid: i64,
    pub name: String,
    pub class: String,
    pub subclass: String,
}

impl ObjectProperties {
    pub fn new_from_node_properties(props: Vec<NodeProperty>) -> Option<Self> {
        let mut prop_iter = props.into_iter();
        let uid = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_i64)
            .next();
        let name_class = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_string)
            .next();
        let subclass = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_string)
            .next();
        if let (Some(uid), Some((name, class)), Some(subclass)) = (
            uid,
            name_class.as_ref().and_then(separate_name_class),
            subclass,
        ) {
            Some(ObjectProperties::new(
                uid,
                name.to_string(),
                class.to_string(),
                subclass,
            ))
        } else {
            None
        }
    }

    fn new(uid: i64, name: String, class: String, subclass: String) -> Self {
        ObjectProperties {
            uid: uid,
            name: name,
            class: class,
            subclass: subclass,
        }
    }
}

/// Returns `Option<(name: String, class: String)>`
fn separate_name_class<'a>(name_class: &'a String) -> Option<(&'a str, &'a str)> {
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
