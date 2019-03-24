use self::property::ObjectProperties;
use fbx_direct::common::OwnedProperty as NodeProperty;
use fbx_direct::reader::{EventReader, FbxEvent, ParserConfig};
use std::io::Read;

pub type NodeData = Option<ObjectProperties>;

pub type Graph = crate::graph::Graph<NodeData, EdgeData>;
pub type Node = crate::graph::Node<NodeData>;
pub type Edge = crate::graph::Edge<EdgeData>;

#[derive(Debug, Default, Clone)]
pub struct EdgeData {
    pub connection_type: Option<String>,
    pub property_name: Option<String>,
}

pub mod filter;
mod property;

struct Context<'a, R: Read> {
    pub graph: &'a mut Graph,
    pub reader: &'a mut EventReader<R>,
}

impl<'a, R: Read> Context<'a, R> {
    pub fn new(graph: &'a mut Graph, reader: &'a mut EventReader<R>) -> Self {
        Context { graph, reader }
    }

    pub fn get_next_node_event(&mut self) -> Option<(String, Vec<NodeProperty>)> {
        get_next_node_event(self.reader)
    }

    pub fn skip_current_node(&mut self) {
        skip_current_node(self.reader)
    }
}

pub fn traverse<R: Read>(graph: &mut Graph, src: &mut R) {
    let parser = &mut ParserConfig::new().ignore_comments(true).create_reader(src);
    let context = &mut Context::new(graph, parser);

    match context.reader.next().unwrap() {
        FbxEvent::StartFbx(_) => {}
        e => {
            panic!("Unexpected event: {:?}", e);
        }
    }

    while let Some((name, _properties)) = context.get_next_node_event() {
        match name.as_ref() {
            "Objects" => traverse_objects(context),
            "Connections" => traverse_connections(context),
            _ => context.skip_current_node(),
        }
    }
}

fn traverse_objects<R: Read>(context: &mut Context<'_, R>) {
    while let Some((name, properties)) = context.get_next_node_event() {
        let obj_props = if let Some(props) = ObjectProperties::new_from_node_properties(properties)
        {
            props
        } else {
            context.skip_current_node();
            continue;
        };
        match name.as_ref() {
            "Pose" => {
                traverse_pose(context, obj_props);
            }
            _ => {
                let node = create_object_node(&obj_props);
                context.graph.add_node(node);
                context.skip_current_node();
            }
        }
    }
}

fn traverse_pose<R: Read>(context: &mut Context<'_, R>, obj_props: ObjectProperties) {
    let mut pose_type = String::new();
    while let Some((name, properties)) = context.get_next_node_event() {
        match name.as_ref() {
            "Type" => {
                if let Some(t) = properties
                    .into_iter()
                    .flat_map(NodeProperty::into_string)
                    .next()
                {
                    pose_type = t;
                }
                context.skip_current_node();
            }
            "PoseNode" => {
                let mut child_id = None;
                while let Some((name, properties)) = context.get_next_node_event() {
                    if name == "Node" {
                        child_id = properties
                            .into_iter()
                            .flat_map(NodeProperty::into_i64)
                            .next();
                    }
                    context.skip_current_node();
                }
                if let Some(child_id) = child_id {
                    let mut edge = Edge::new(obj_props.uid, child_id);
                    edge.data.connection_type = Some("Pose".to_string());
                    context.graph.add_edge(edge);
                }
            }
            _ => context.skip_current_node(),
        }
    }
    let _ = pose_type;
    let node = create_object_node(&obj_props);
    context.graph.add_node(node);
}

fn traverse_connections<R: Read>(context: &mut Context<'_, R>) {
    while let Some((name, properties)) = context.get_next_node_event() {
        if name != "C" {
            context.skip_current_node();
            continue;
        }
        let mut prop_iter = properties.into_iter();
        let connection_type = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_string)
            .next();
        let child_uid = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_i64)
            .next();
        let parent_uid = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_i64)
            .next();
        let property_name = prop_iter
            .next()
            .into_iter()
            .flat_map(NodeProperty::into_string)
            .next();
        if let (Some(connection_type), Some(child_uid), Some(parent_uid)) =
            (connection_type, child_uid, parent_uid)
        {
            let mut edge = Edge::new(parent_uid, child_uid);
            edge.data.connection_type = Some(connection_type);
            if let Some(prop_name) = property_name {
                edge.styles.insert("label".to_string(), prop_name.clone());
                edge.data.property_name = Some(prop_name);
            }
            context.graph.add_edge(edge);
        }
        context.skip_current_node();
    }
}

pub fn create_object_node(obj_props: &ObjectProperties) -> Node {
    let mut node = Node::new_with_data(obj_props.uid, Some(obj_props.clone()));
    let label = format!(
        "{}::{}\\n{}\\n{}",
        obj_props.class, obj_props.name, obj_props.subclass, obj_props.uid
    );
    node.styles.insert("label".to_string(), label);
    node
}

/// Get a start or end of node event.
///
/// Returns `Ok(Some((name, properties)))` if got a start of node event,
/// `Ok(None)` if got an end of a node or an event,
/// and `Error` if got an unexpected event (such as a start of FBX document).
fn get_next_node_event<R: Read>(
    reader: &mut EventReader<R>,
) -> Option<(String, Vec<NodeProperty>)> {
    loop {
        match reader.next().unwrap() {
            FbxEvent::StartNode { name, properties } => {
                return Some((name, properties));
            }
            FbxEvent::EndNode | FbxEvent::EndFbx => {
                return None;
            }
            FbxEvent::Comment(_) => {}
            ref e => {
                // Unreachable if programmers use this function properly.
                panic!("Oops! Expected `StartNode`, `EndNode` or `EndFbx`, but got `{:?}`, this may not be what the programmer(s) wanted...", e);
            }
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
            }
            FbxEvent::EndNode => {
                if depth == 0 {
                    return;
                }
                depth -= 1;
            }
            FbxEvent::EndFbx => {
                // Unreachable if programmers use this function properly.
                panic!("Skipped to the end of FBX (but it is not expected)");
            }
            _ => {}
        }
    }
}
