use std::io::Read;

use fbxcel::pull_parser::any::AnyParser;

pub use self::property::ObjectProperties;

pub mod filter;
mod property;
pub mod v7400;

pub type NodeData = Option<ObjectProperties>;

pub type Graph = crate::graph::Graph<NodeData, EdgeData>;
pub type Node = crate::graph::Node<NodeData>;
pub type Edge = crate::graph::Edge<EdgeData>;

#[derive(Debug, Default, Clone)]
pub struct EdgeData {
    pub connection_type: Option<String>,
    pub property_name: Option<String>,
}

pub fn traverse(graph: &mut Graph, src: impl Read) {
    match AnyParser::from_reader(src).expect("Failed to create FBX parser") {
        AnyParser::V7400(parser) => v7400::traverse(graph, parser),
        parser => panic!("Unknown parser version: {:?}", parser.parser_version()),
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
