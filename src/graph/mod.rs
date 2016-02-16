use std::collections::{HashSet, HashMap};
use std::io::Write;
use std::io;

#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone> {
    pub name: String,
    pub graph_styles: HashMap<String, String>,
    pub node_styles: HashMap<String, String>,
    pub nodes: HashMap<i64, Node<N>>,
    pub edges: Vec<Edge<E>>,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Graph {
            name: name.into(),
            graph_styles: Default::default(),
            node_styles: Default::default(),
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: Node<N>) -> Option<Node<N>> {
        self.nodes.insert(node.id, node)
    }

    pub fn add_edge(&mut self, edge: Edge<E>) {
        self.edges.push(edge);
    }

    pub fn output_all<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(self.print_beginning(out));
        // Print nodes
        for (_, n) in &self.nodes {
            try!(n.print(out));
        }
        // Print edges
        for e in &self.edges {
            try!(e.print(out));
        }
        try!(self.print_ending(out));
        Ok(())
    }

    pub fn output_visible_nodes<W: Write>(&self, out: &mut W, print_unregistered_nodes: bool) -> io::Result<()> {
        try!(self.print_beginning(out));
        // Print visible nodes
        for (_, n) in self.nodes.iter().filter(|&(_, n)| n.is_visible()) {
            try!(n.print(out));
        }
        // Print edges
        for e in &self.edges {
            let parent_is_visible = self.nodes.get(&e.parent).map_or(print_unregistered_nodes, |n| n.is_visible());
            let child_is_visible = self.nodes.get(&e.child).map_or(print_unregistered_nodes, |n| n.is_visible());
            if parent_is_visible && child_is_visible {
                try!(e.print(out));
            }
        }
        try!(self.print_ending(out));
        Ok(())
    }

    pub fn print_beginning<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(writeln!(out, "digraph \"{}\" {{", self.name));

        // Print graph settings
        if self.graph_styles.len() > 0 {
            let mut print_comma = false;
            try!(writeln!(out, "\tgraph ["));
            for (key, value) in &self.graph_styles {
                if print_comma {
                    try!(write!(out, "\n, "));
                }
                try!(write!(out, "\t\t{}=\"{}\"", style_escape(key), style_escape(value)));
                print_comma = true;
            }
            try!(writeln!(out, "\n\t]"));
        }

        // Print node settings
        if self.node_styles.len() > 0 {
            let mut print_comma = false;
            try!(writeln!(out, "\tnode ["));
            for (key, value) in &self.node_styles {
                if print_comma {
                    try!(write!(out, "\n, "));
                }
                try!(write!(out, "\t\t{}=\"{}\"", style_escape(key), style_escape(value)));
                print_comma = true;
            }
            try!(writeln!(out, "\n\t]"));
        }
        Ok(())
    }

    pub fn print_ending<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(writeln!(out, "}}"));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Node<T: Clone> {
    pub id: i64,
    pub visible: bool,
    pub styles: HashMap<String, String>,
    pub data: T,
}

impl<T: Clone+Default> Node<T> {
    pub fn new(id: i64) -> Self {
        Node::<T>::new_with_data(id, Default::default())
    }
}

impl<T: Clone> Node<T> {
    pub fn new_with_data(id: i64, data: T) -> Self {
        Node {
            id: id,
            visible: true,
            styles: Default::default(),
            data: data
        }
    }

    pub fn print<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(write!(out, "\t{}", self.id));
        if self.styles.len() > 0 {
            let mut print_comma = false;
            try!(write!(out, " ["));
            for (key, value) in &self.styles {
                if print_comma {
                    try!(write!(out, ", "));
                }
                try!(write!(out, "{}=\"{}\"", style_escape(key), style_escape(value)));
                print_comma = true;
            }
            try!(write!(out, "]"));
        }
        try!(write!(out, "\n"));
        Ok(())
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[derive(Debug, Clone)]
pub struct Edge<T: Clone> {
    pub parent: i64,
    pub child: i64,
    pub styles: HashMap<String, String>,
    pub data: T,
}

impl<T: Clone+Default> Edge<T> {
    pub fn new(parent: i64, child: i64) -> Self {
        Edge::<T>::new_with_data(parent, child, Default::default())
    }
}

impl<T: Clone> Edge<T> {
    pub fn new_with_data(parent: i64, child: i64, data: T) -> Self {
        Edge {
            parent: parent,
            child: child,
            styles: Default::default(),
            data: data,
        }
    }

    pub fn print<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(write!(out, "\t{} -> {}", self.parent, self.child));
        if self.styles.len() > 0 {
            let mut print_comma = false;
            try!(write!(out, " ["));
            for (key, value) in &self.styles {
                if print_comma {
                    try!(write!(out, ", "));
                }
                try!(write!(out, "{}=\"{}\"", style_escape(key), style_escape(value)));
                print_comma = true;
            }
            try!(write!(out, "]"));
        }
        try!(write!(out, "\n"));
        Ok(())
    }
}

fn style_escape(raw: &str) -> String {
    raw.replace("\"", "\\\"")
}
