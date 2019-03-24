use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone> {
    pub name: String,
    pub graph_styles: HashMap<String, String>,
    pub node_styles: HashMap<String, String>,
    pub edge_styles: HashMap<String, String>,
    pub nodes: BTreeMap<i64, Node<N>>,
    pub edges: Vec<Edge<E>>,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Graph {
            name: name.into(),
            graph_styles: Default::default(),
            node_styles: Default::default(),
            edge_styles: Default::default(),
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

    pub fn map_ascendant<I, F>(&mut self, targets: I, fun: F)
    where
        I: IntoIterator<Item = i64>,
        F: Fn(&mut Node<N>),
    {
        let mut done = HashSet::new();
        // Get parents of `targets`.
        let mut undone_next = targets
            .into_iter()
            .flat_map(|i| {
                self.edges
                    .iter()
                    .filter(|e| e.child == i)
                    .map(|e| e.parent)
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect::<HashSet<i64>>();
        loop {
            let undone_current = undone_next;
            undone_next = HashSet::new();
            for target in undone_current {
                if done.contains(&target) {
                    continue;
                }
                // Process current node.
                self.nodes.get_mut(&target).map(&fun);
                // Queue parents of the `target`.
                for parent in self
                    .edges
                    .iter()
                    .filter(|e| e.child == target)
                    .map(|e| e.parent)
                    .filter(|p| !done.contains(p))
                {
                    undone_next.insert(parent);
                }
                done.insert(target);
            }
            if undone_next.is_empty() {
                break;
            }
        }
    }

    pub fn map_descendant<I, F>(&mut self, targets: I, fun: F)
    where
        I: IntoIterator<Item = i64>,
        F: Fn(&mut Node<N>),
    {
        let mut done = HashSet::new();
        // Get children of `targets`.
        let mut undone_next = targets
            .into_iter()
            .flat_map(|i| {
                self.edges
                    .iter()
                    .filter(|e| e.parent == i)
                    .map(|e| e.child)
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect::<HashSet<i64>>();
        loop {
            let undone_current = undone_next;
            undone_next = HashSet::new();
            for target in undone_current {
                if done.contains(&target) {
                    continue;
                }
                // Process current node.
                self.nodes.get_mut(&target).map(&fun);
                // Queue children of the `target`.
                for parent in self
                    .edges
                    .iter()
                    .filter(|e| e.parent == target)
                    .map(|e| e.child)
                    .filter(|p| !done.contains(p))
                {
                    undone_next.insert(parent);
                }
                done.insert(target);
            }
            if undone_next.is_empty() {
                break;
            }
        }
    }

    pub fn map_parents<I, F>(&mut self, targets: I, fun: F)
    where
        I: IntoIterator<Item = i64>,
        F: Fn(&mut Node<N>),
    {
        // Get parents of `targets`.
        let targets = targets
            .into_iter()
            .flat_map(|i| {
                self.edges
                    .iter()
                    .filter(|e| e.child == i)
                    .map(|e| e.parent)
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect::<Vec<i64>>();
        for target in targets {
            // Process current node.
            self.nodes.get_mut(&target).map(&fun);
        }
    }

    pub fn map_children<I, F>(&mut self, targets: I, fun: F)
    where
        I: IntoIterator<Item = i64>,
        F: Fn(&mut Node<N>),
    {
        // Get children of `targets`.
        let targets = targets
            .into_iter()
            .flat_map(|i| {
                self.edges
                    .iter()
                    .filter(|e| e.parent == i)
                    .map(|e| e.child)
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect::<Vec<i64>>();
        for target in targets {
            // Process current node.
            self.nodes.get_mut(&target).map(&fun);
        }
    }

    pub fn output_all<W: Write>(&self, out: &mut W) -> io::Result<()> {
        self.print_beginning(out)?;
        // Print nodes
        for (_, n) in &self.nodes {
            n.print(out)?;
        }
        // Print edges
        for e in &self.edges {
            e.print(out)?;
        }
        self.print_ending(out)?;
        Ok(())
    }

    pub fn output_visible_nodes<W: Write>(
        &self,
        out: &mut W,
        print_unregistered_nodes: bool,
    ) -> io::Result<()> {
        self.print_beginning(out)?;
        // Print visible nodes
        for (_, n) in self.nodes.iter().filter(|&(_, n)| n.is_visible()) {
            n.print(out)?;
        }
        // Print edges
        for e in &self.edges {
            let parent_is_visible = self.nodes.get(&e.parent).map(|n| n.is_visible());
            let child_is_visible = self.nodes.get(&e.child).map(|n| n.is_visible());
            if (parent_is_visible.is_some() || child_is_visible.is_some())
                && (parent_is_visible.unwrap_or(print_unregistered_nodes)
                    && child_is_visible.unwrap_or(print_unregistered_nodes))
            {
                e.print(out)?;
            }
        }
        self.print_ending(out)?;
        Ok(())
    }

    pub fn print_beginning<W: Write>(&self, out: &mut W) -> io::Result<()> {
        writeln!(out, "digraph \"{}\" {{", self.name)?;

        // Print graph settings.
        if self.graph_styles.len() > 0 {
            let mut print_comma = false;
            writeln!(out, "\tgraph [")?;
            for (key, value) in &self.graph_styles {
                if print_comma {
                    write!(out, "\n, ")?;
                }
                write!(out, "\t\t{}=\"{}\"", style_escape(key), style_escape(value))?;
                print_comma = true;
            }
            writeln!(out, "\n\t]")?;
        }

        // Print node settings.
        if self.node_styles.len() > 0 {
            let mut print_comma = false;
            writeln!(out, "\tnode [")?;
            for (key, value) in &self.node_styles {
                if print_comma {
                    write!(out, "\n, ")?;
                }
                write!(out, "\t\t{}=\"{}\"", style_escape(key), style_escape(value))?;
                print_comma = true;
            }
            writeln!(out, "\n\t]")?;
        }

        // Print edge settings.
        if self.edge_styles.len() > 0 {
            let mut print_comma = false;
            writeln!(out, "\tedge [")?;
            for (key, value) in &self.edge_styles {
                if print_comma {
                    write!(out, "\n, ")?;
                }
                write!(out, "\t\t{}=\"{}\"", style_escape(key), style_escape(value))?;
                print_comma = true;
            }
            writeln!(out, "\n\t]")?;
        }
        Ok(())
    }

    pub fn print_ending<W: Write>(&self, out: &mut W) -> io::Result<()> {
        writeln!(out, "}}")?;
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

impl<T: Clone + Default> Node<T> {
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
            data: data,
        }
    }

    pub fn print<W: Write>(&self, out: &mut W) -> io::Result<()> {
        write!(out, "\t{}", self.id)?;
        if self.styles.len() > 0 {
            let mut print_comma = false;
            write!(out, " [")?;
            for (key, value) in &self.styles {
                if print_comma {
                    write!(out, ", ")?;
                }
                write!(out, "{}=\"{}\"", style_escape(key), style_escape(value))?;
                print_comma = true;
            }
            write!(out, "]")?;
        }
        write!(out, "\n")?;
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

impl<T: Clone + Default> Edge<T> {
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
        write!(out, "\t{} -> {}", self.parent, self.child)?;
        if self.styles.len() > 0 {
            let mut print_comma = false;
            write!(out, " [")?;
            for (key, value) in &self.styles {
                if print_comma {
                    write!(out, ", ")?;
                }
                write!(out, "{}=\"{}\"", style_escape(key), style_escape(value))?;
                print_comma = true;
            }
            write!(out, "]")?;
        }
        write!(out, "\n")?;
        Ok(())
    }
}

fn style_escape(raw: &str) -> String {
    raw.replace("\"", "\\\"")
}
