use std::collections::HashMap;
use std::io::Write;
use std::io;

#[derive(Debug, Clone)]
pub struct Graph {
    pub name: String,
    pub graph_styles: Vec<String>,
    pub node_styles: Vec<String>,
    pub nodes: HashMap<i64, Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Graph {
            name: name.into(),
            graph_styles: vec![],
            node_styles: vec![],
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> Option<Node> {
        self.nodes.insert(node.id, node)
    }

    pub fn add_edge(&mut self, edge: Edge) {
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
            for style in &self.graph_styles {
                if print_comma {
                    try!(write!(out, "\n, "));
                }
                try!(write!(out, "\t\t{}", style));
                print_comma = true;
            }
            try!(writeln!(out, "\n\t]"));
        }

        // Print node settings
        if self.node_styles.len() > 0 {
            let mut print_comma = false;
            try!(writeln!(out, "\tnode ["));
            for style in &self.node_styles {
                if print_comma {
                    try!(write!(out, "\n, "));
                }
                try!(write!(out, "\t\t{}", style));
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
pub struct Node {
    pub id: i64,
    pub visible: bool,
    pub styles: Vec<String>,
}

impl Node {
    pub fn new(id: i64) -> Self {
        Node {
            id: id,
            visible: true,
            styles: vec![],
        }
    }

    pub fn print<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(write!(out, "\t{}", self.id));
        if self.styles.len() > 0 {
            let mut print_comma = false;
            try!(write!(out, " ["));
            for style in &self.styles {
                if print_comma {
                    try!(write!(out, ", "));
                }
                try!(write!(out, "{}", style));
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
pub struct Edge {
    pub parent: i64,
    pub child: i64,
    pub styles: Vec<String>,
}

impl Edge {
    pub fn new(parent: i64, child: i64) -> Self {
        Edge {
            parent: parent,
            child: child,
            styles: vec![],
        }
    }

    pub fn print<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(write!(out, "\t{} -> {}", self.parent, self.child));
        if self.styles.len() > 0 {
            let mut print_comma = false;
            try!(write!(out, " ["));
            for style in &self.styles {
                if print_comma {
                    try!(write!(out, ", "));
                }
                try!(write!(out, "{}", style));
                print_comma = true;
            }
            try!(write!(out, "]"));
        }
        try!(write!(out, "\n"));
        Ok(())
    }
}
