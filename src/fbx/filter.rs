use crate::fbx::{Edge, Graph, Node};
use regex::{self, Regex};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Filters {
    pub graph_styles: HashMap<String, String>,
    pub node_styles: HashMap<String, String>,
    pub edge_styles: HashMap<String, String>,
    pub node_operations: BTreeMap<String, Vec<NodeOperation>>,
    pub edge_operations: BTreeMap<String, Vec<EdgeOperation>>,
    pub node_filters: Vec<NodeFilter>,
    pub edge_filters: Vec<EdgeFilter>,
    pub show_implicit_nodes: Option<bool>,
}

impl Filters {
    pub fn apply(&self, graph: &mut Graph) {
        for (name, value) in &self.node_styles {
            graph.node_styles.insert(name.clone(), value.clone());
        }
        for (name, value) in &self.edge_styles {
            graph.edge_styles.insert(name.clone(), value.clone());
        }
        for (name, value) in &self.graph_styles {
            graph.graph_styles.insert(name.clone(), value.clone());
        }

        {
            // Compile node filter conditions.
            let node_conditions = self
                .node_filters
                .iter()
                .map(|f| Ok::<_, regex::Error>((f.condition.compile()?, &f.operations)))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            // Apply each condition to all nodes.
            for &(ref cond, op_names) in &node_conditions {
                let target_uids = graph
                    .nodes
                    .iter()
                    .filter(|&(_, node)| cond.is_match(node))
                    .map(|(&uid, _)| uid)
                    .collect::<Vec<_>>();
                for uid in target_uids {
                    self.apply_node_operations(uid, graph, op_names);
                }
            }
        }
        {
            // Compile edge filter conditions.
            let edge_conditions = self
                .edge_filters
                .iter()
                .map(|f| Ok::<_, regex::Error>((f.condition.compile()?, &f.operations)))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            // Apply each condition to all edges.
            for &(ref cond, op_names) in &edge_conditions {
                let (nodes, edges) = (&mut graph.nodes, &mut graph.edges);
                let target_edges = edges
                    .iter_mut()
                    .filter(|edge| cond.is_match(edge, nodes))
                    .collect::<Vec<_>>();
                for target_edge in target_edges {
                    self.apply_edge_operation(target_edge, nodes, op_names);
                }
            }
        }
    }

    fn apply_node_operations(&self, id: i64, graph: &mut Graph, ops: &[String]) {
        for ops in ops.iter().filter_map(|s| self.node_operations.get(s)) {
            for op in ops {
                match op.name.as_ref() {
                    "update-attr" => {
                        for arg in &op.args {
                            if arg.len() < 2 {
                                continue;
                            }
                            let name = arg[0].clone();
                            let value = arg[1].clone();
                            graph
                                .nodes
                                .get_mut(&id)
                                .map(|n| n.styles.insert(name, value));
                        }
                    }
                    "remove-attr" => {
                        if let Some(args) = op.args.get(0) {
                            for name in args {
                                graph.nodes.get_mut(&id).map(|n| n.styles.remove(name));
                            }
                        }
                    }
                    "hide" | "show" => {
                        let visibility = op.name == "show";
                        if let Some(args) = op.args.get(0) {
                            for target in args {
                                match target.as_ref() {
                                    "self" => {
                                        if let Some(n) = graph.nodes.get_mut(&id) {
                                            n.visible = visibility;
                                        }
                                    }
                                    "ascendant" => {
                                        graph.map_ascendant(Some(id), |n| n.visible = visibility);
                                    }
                                    "descendant" => {
                                        graph.map_descendant(Some(id), |n| n.visible = visibility);
                                    }
                                    "parents" => {
                                        graph.map_parents(Some(id), |n| n.visible = visibility);
                                    }
                                    "children" => {
                                        graph.map_children(Some(id), |n| n.visible = visibility);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn apply_edge_operation(
        &self,
        edge: &mut Edge,
        _nodes: &mut BTreeMap<i64, Node>,
        ops: &[String],
    ) {
        for ops in ops.iter().filter_map(|s| self.edge_operations.get(s)) {
            for op in ops {
                match op.name.as_ref() {
                    "update-attr" => {
                        for arg in &op.args {
                            if arg.len() < 2 {
                                continue;
                            }
                            let name = arg[0].clone();
                            let value = arg[1].clone();
                            edge.styles.insert(name, value);
                        }
                    }
                    "remove-attr" => {
                        if let Some(args) = op.args.get(0) {
                            for name in args {
                                edge.styles.remove(name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeOperation {
    pub name: String,
    pub args: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdgeOperation {
    pub name: String,
    pub args: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeFilter {
    pub condition: NodeFilterCondition,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeFilterCondition {
    pub class: Option<String>,
    pub subclass: Option<String>,
    pub name: Option<String>,
    pub uid: Option<String>,
}

impl NodeFilterCondition {
    pub fn compile(&self) -> Result<CompiledNodeFilterCondition, regex::Error> {
        let class = if let Some(ref s) = self.class {
            Some(Regex::new(s)?)
        } else {
            None
        };
        let subclass = if let Some(ref s) = self.subclass {
            Some(Regex::new(s)?)
        } else {
            None
        };
        let name = if let Some(ref s) = self.name {
            Some(Regex::new(s)?)
        } else {
            None
        };
        let uid = if let Some(ref s) = self.uid {
            Some(Regex::new(s)?)
        } else {
            None
        };
        Ok(CompiledNodeFilterCondition {
            class,
            subclass,
            name,
            uid,
        })
    }
}

pub struct CompiledNodeFilterCondition {
    pub class: Option<Regex>,
    pub subclass: Option<Regex>,
    pub name: Option<Regex>,
    pub uid: Option<Regex>,
}

impl CompiledNodeFilterCondition {
    pub fn is_match(&self, node: &Node) -> bool {
        if let Some(ref data) = node.data {
            if let Some(ref re) = self.class {
                if !re.is_match(&data.class) {
                    return false;
                }
            }
            if let Some(ref re) = self.subclass {
                if !re.is_match(&data.subclass) {
                    return false;
                }
            }
            if let Some(ref re) = self.name {
                if !re.is_match(&data.name) {
                    return false;
                }
            }
        } else if self.class.is_some() || self.subclass.is_some() || self.name.is_some() {
            return false;
        }
        if let Some(ref re) = self.uid {
            if !re.is_match(&node.id.to_string()) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdgeFilter {
    pub condition: EdgeFilterCondition,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdgeFilterCondition {
    pub src_condition: Option<NodeFilterCondition>,
    pub dst_condition: Option<NodeFilterCondition>,
    pub connection_type: Option<String>,
    pub property_name: Option<String>,
}

impl EdgeFilterCondition {
    pub fn compile(&self) -> Result<CompiledEdgeFilterCondition, regex::Error> {
        let src_condition = if let Some(ref cond) = self.src_condition {
            Some(cond.compile()?)
        } else {
            None
        };
        let dst_condition = if let Some(ref cond) = self.dst_condition {
            Some(cond.compile()?)
        } else {
            None
        };
        let connection_type = if let Some(ref s) = self.connection_type {
            Some(Regex::new(s)?)
        } else {
            None
        };
        let property_name = if let Some(ref s) = self.property_name {
            Some(Regex::new(s)?)
        } else {
            None
        };
        Ok(CompiledEdgeFilterCondition {
            src_condition,
            dst_condition,
            connection_type,
            property_name,
        })
    }
}

pub struct CompiledEdgeFilterCondition {
    pub src_condition: Option<CompiledNodeFilterCondition>,
    pub dst_condition: Option<CompiledNodeFilterCondition>,
    pub connection_type: Option<Regex>,
    pub property_name: Option<Regex>,
}

impl CompiledEdgeFilterCondition {
    pub fn is_match(&self, edge: &Edge, nodes: &BTreeMap<i64, Node>) -> bool {
        if let Some(ref cond) = self.src_condition {
            if let Some(src) = nodes.get(&edge.parent) {
                if !cond.is_match(src) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref cond) = self.dst_condition {
            if let Some(dst) = nodes.get(&edge.child) {
                if !cond.is_match(dst) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref re) = self.connection_type {
            if let Some(ref con_type) = edge.data.connection_type {
                if !re.is_match(con_type) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref re) = self.property_name {
            if let Some(ref prop_name) = edge.data.property_name {
                if !re.is_match(prop_name) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
