use std::io::{Read, Write};
use fbx_direct::reader::{FbxEvent, EventReader, ParserConfig};
use fbx_direct::common::OwnedProperty as NodeProperty;

struct Context<'a, W: 'a+Write, R: 'a+Read> {
    pub dest: &'a mut W,
    pub reader: &'a mut EventReader<R>,
}

impl<'a, W: Write, R: Read> Context<'a, W, R> {
    pub fn new(dest: &'a mut W, reader: &'a mut EventReader<R>) -> Self {
        Context {
            dest: dest,
            reader: reader,
        }
    }

    pub fn get_next_node_event(&mut self) -> Option<(String, Vec<NodeProperty>)> {
        get_next_node_event(self.reader)
    }

    pub fn skip_current_node(&mut self) {
        skip_current_node(self.reader)
    }
}

pub fn traverse<W: Write, R: Read>(dest: &mut W, src: &mut R) {
    let parser = &mut ParserConfig::new().ignore_comments(true).create_reader(src);
    let context = &mut Context::new(dest, parser);

    match context.reader.next().unwrap() {
        FbxEvent::StartFbx(_) => {},
        e => {
            panic!("Unexpected event: {:?}", e);
        },
    }

    while let Some((name, _properties)) = context.get_next_node_event() {
        match name.as_ref() {
            "Objects" => traverse_objects(context),
            "Connections" => traverse_connections(context),
            _ => context.skip_current_node(),
        }
    }
}

fn traverse_objects<W: Write, R: Read>(context: &mut Context<W, R>) {
    while let Some((name, properties)) = context.get_next_node_event() {
        context.skip_current_node();
    }
}

pub enum LinkEndType {
    Object,
    Property,
}

pub struct ConnectionType {
    pub parent: LinkEndType,
    pub child: LinkEndType,
}

impl ConnectionType {
    pub fn from_string(s: &str) -> Option<Self> {
        Some(match s {
            "OO" => ConnectionType {
                parent: LinkEndType::Object,
                child: LinkEndType::Object,
            },
            "OP" => ConnectionType {
                parent: LinkEndType::Object,
                child: LinkEndType::Property,
            },
            "PO" => ConnectionType {
                parent: LinkEndType::Property,
                child: LinkEndType::Object,
            },
            "PP" => ConnectionType {
                parent: LinkEndType::Property,
                child: LinkEndType::Property,
            },
            _ => {
                return None;
            },
        })
    }
}

fn traverse_connections<W: Write, R: Read>(context: &mut Context<W, R>) {
    while let Some((name, properties)) = context.get_next_node_event() {
        if name != "C" {
            context.skip_current_node();
            continue;
        }
        let mut prop_iter = properties.into_iter();
        let connection_type = prop_iter.next().and_then(NodeProperty::into_string).and_then(|v| ConnectionType::from_string(&v));
        let child_uid = prop_iter.next().and_then(NodeProperty::into_i64);
        let parent_uid = prop_iter.next().and_then(NodeProperty::into_i64);
        let property_name = prop_iter.next().and_then(NodeProperty::into_string);
        if let (Some(connection_type), Some(child_uid), Some(parent_uid)) = (connection_type, child_uid, parent_uid) {
            writeln!(context.dest, "\t{} -> {};", parent_uid, child_uid).unwrap();
        }
        context.skip_current_node();
    }
}

/// Get a start or end of node event.
///
/// Returns `Ok(Some((name, properties)))` if got a start of node event,
/// `Ok(None)` if got an end of a node or an event,
/// and `Error` if got an unexpected event (such as a start of FBX document).
fn get_next_node_event<R: Read>(reader: &mut EventReader<R>) -> Option<(String, Vec<NodeProperty>)> {
    loop {
        match reader.next().unwrap() {
            FbxEvent::StartNode { name, properties } => {
                return Some((name, properties));
            },
            FbxEvent::EndNode | FbxEvent::EndFbx => {
                return None;
            },
            FbxEvent::Comment(_) => {},
            ref e => {
                // Unreachable if programmers use this function properly.
                panic!("Oops! Expected `StartNode`, `EndNode` or `EndFbx`, but got `{:?}`, this may not be what the programmer(s) wanted...", e);
            },
        }
    }
}

/// Skip to the end of the current node.
fn skip_current_node<R: Read>(reader: &mut EventReader<R>) {
    let mut depth = 0_usize;
    loop {
        match reader.next().unwrap() {
            FbxEvent::StartNode { .. } => {
                depth += 1;
            },
            FbxEvent::EndNode => {
                if depth == 0 {
                    return;
                }
                depth -= 1;
            },
            FbxEvent::EndFbx => {
                // Unreachable if programmers use this function properly.
                panic!("Skipped to the end of FBX (but it is not expected)");
            },
            _ => {}
        }
    }
}
