extern crate docopt;
extern crate fbx_direct;
extern crate rustc_serialize;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use docopt::Docopt;

mod fbx;
pub mod graph;

const USAGE: &'static str = "
Visualize FBX objects dependency.

Usage:
    fbx_objects_depviz <fbx-name> [--output=<dot-name>]

Options:
    --output=<dot-name>     Output filename.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_fbx_name: String,
    flag_output: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let mut graph = graph::Graph::new(args.arg_fbx_name.clone());
    graph.node_styles.push("shape=box".to_string());

    // Add implicit root node.
    {
        let root = graph::Node::new(0);
        graph.add_node(root);
    }

    let mut src = BufReader::new(File::open(&args.arg_fbx_name).unwrap());
    let mut out: BufWriter<_> = BufWriter::new(if let Some(ref out_name) = args.flag_output {
        Box::new(File::create(&out_name).unwrap()) as Box<Write>
    } else {
        Box::new(::std::io::stdout()) as Box<Write>
    });

    fbx::traverse(&mut graph, &mut src);

    graph.output_visible_nodes(&mut out, false).unwrap();
}
