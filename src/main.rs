extern crate docopt;
extern crate fbx_direct;
extern crate regex;
extern crate rustc_serialize;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use docopt::Docopt;
use rustc_serialize::json;

mod fbx;
pub mod graph;

const USAGE: &'static str = "
Visualize FBX objects dependency.

Usage:
    fbx_objects_depviz <fbx-name> [--output=<dot-name>] [--filter=<filter-json>]

Options:
    --output=<dot-name>     Output filename.
    --filter=<filter-json>  Filter written in json.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_fbx_name: String,
    flag_output: Option<String>,
    flag_filter: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let mut src = BufReader::new(File::open(&args.arg_fbx_name).unwrap());
    let mut out: BufWriter<_> = BufWriter::new(if let Some(ref out_name) = args.flag_output {
        Box::new(File::create(&out_name).unwrap()) as Box<Write>
    } else {
        Box::new(::std::io::stdout()) as Box<Write>
    });

    let mut graph = fbx::Graph::new(args.arg_fbx_name.clone());

    // Add implicit root node.
    graph.add_node(fbx::Node::new(0));

    fbx::traverse(&mut graph, &mut src);

    if let Some(ref filter_filename) = args.flag_filter {
        let filters: fbx::filter::Filters = {
            use std::io::Read;
            let mut filter_json_str = String::new();
            File::open(filter_filename).unwrap().read_to_string(&mut filter_json_str).unwrap();
            json::decode(&filter_json_str).unwrap()
        };
        filters.apply(&mut graph);
        graph.output_visible_nodes(&mut out, filters.show_implicit_nodes.unwrap_or(false)).unwrap();
    } else {
        graph.output_all(&mut out).unwrap();
    }
}
