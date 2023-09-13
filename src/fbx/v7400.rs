use std::io::Read;

use crate::fbx::{create_object_node, Edge, Graph, ObjectProperties};

use fbxcel::{
    low::v7400::AttributeValue,
    pull_parser::v7400::{attribute::loaders::DirectLoader, Attributes, Event, Parser},
};

pub fn traverse<R: Read>(graph: &mut Graph, mut parser: Parser<R>) {
    assert!(!parser.is_used());
    loop {
        match parser.next_event().expect("Failed to parse") {
            Event::StartNode(node) => match node.name() {
                "Objects" => traverse_objects(graph, &mut parser),
                "Connections" => traverse_connections(graph, &mut parser),
                _ => parser.skip_current_node().unwrap(),
            },
            Event::EndNode => unreachable!(),
            Event::EndFbx(_) => break,
        }
    }
}

fn traverse_objects<R: Read>(graph: &mut Graph, parser: &mut Parser<R>) {
    loop {
        match parser.next_event().expect("Failed to parse") {
            Event::StartNode(node) => {
                let is_pose = node.name() == "Pose";
                let props = match ObjectProperties::from_attrs7400(node.attributes()) {
                    Some(v) => v,
                    None => {
                        parser.skip_current_node().unwrap();
                        continue;
                    }
                };
                if is_pose {
                    traverse_pose(graph, parser, &props);
                } else {
                    let node = create_object_node(&props);
                    graph.add_node(node);
                    parser.skip_current_node().unwrap();
                }
            }
            Event::EndNode => break,
            Event::EndFbx(_) => unreachable!(),
        }
    }
}

fn traverse_pose<R: Read>(graph: &mut Graph, parser: &mut Parser<R>, props: &ObjectProperties) {
    let mut pose_type = String::new();
    loop {
        match parser.next_event().expect("Failed to parse") {
            Event::StartNode(node) => match node.name() {
                "Type" => {
                    if let Some(AttributeValue::String(s)) =
                        node.attributes().load_next(DirectLoader).unwrap()
                    {
                        pose_type = s;
                    }
                    parser.skip_current_node().unwrap();
                }
                "PoseNode" => {
                    let mut child_id = None;
                    'pose_node: loop {
                        match parser.next_event().expect("Failed to parse") {
                            Event::StartNode(node) => {
                                if node.name() == "Node" {
                                    child_id = node
                                        .attributes()
                                        .load_next(DirectLoader)
                                        .unwrap()
                                        .and_then(|attr| attr.get_i64());
                                }
                                parser.skip_current_node().unwrap();
                            }
                            Event::EndNode => break 'pose_node,
                            Event::EndFbx(_) => unreachable!(),
                        }
                    }
                    if let Some(child_id) = child_id {
                        let mut edge = Edge::new(props.uid, child_id);
                        edge.data.connection_type = Some("Pose".to_owned());
                        graph.add_edge(edge);
                    }
                }
                _ => parser.skip_current_node().unwrap(),
            },
            Event::EndNode => break,
            Event::EndFbx(_) => unreachable!(),
        }
    }
    let _ = pose_type;
    let node = create_object_node(&props);
    graph.add_node(node);
}

fn traverse_connections<R: Read>(graph: &mut Graph, parser: &mut Parser<R>) {
    loop {
        match parser.next_event().expect("Failed to parse") {
            Event::StartNode(node) => {
                if node.name() != "C" {
                    parser.skip_current_node().unwrap();
                    continue;
                }
                if let Some((connection_type, child_uid, parent_uid, property_name)) =
                    load_connection(node.attributes())
                {
                    let mut edge = Edge::new(parent_uid, child_uid);
                    edge.data.connection_type = Some(connection_type);
                    if let Some(prop_name) = property_name {
                        edge.styles.insert("label".to_string(), prop_name.clone());
                        edge.data.property_name = Some(prop_name);
                    }
                    graph.add_edge(edge);
                }
                parser.skip_current_node().unwrap();
            }
            Event::EndNode => break,
            Event::EndFbx(_) => unreachable!(),
        }
    }
}

fn load_connection<R: Read>(
    attrs: Attributes<'_, R>,
) -> Option<(String, i64, i64, Option<String>)> {
    let mut attrs = attrs.into_iter(std::iter::repeat(DirectLoader));
    let connection_type = attrs.next()?.unwrap().get_string()?.to_owned();
    let child_uid = attrs.next()?.unwrap().get_i64()?;
    let parent_uid = attrs.next()?.unwrap().get_i64()?;
    let property_name = attrs
        .next()
        .and_then(|attr| attr.unwrap().get_string().map(Into::into));
    Some((connection_type, child_uid, parent_uid, property_name))
}
