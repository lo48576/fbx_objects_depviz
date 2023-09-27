use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;

mod fbx;
pub mod graph;

#[derive(Debug, Parser)]
struct CliOpt {
    /// FBX file path
    #[clap(name = "fbx-name")]
    fbx_path: PathBuf,
    /// Output dot file path
    #[clap(long = "output")]
    output: Option<PathBuf>,
    /// Filter json file path
    #[clap(long = "filter")]
    filter: Option<PathBuf>,
}

fn main() {
    let opt = CliOpt::parse();

    let mut src = BufReader::new(File::open(&opt.fbx_path).unwrap());
    let mut out: BufWriter<_> = BufWriter::new(if let Some(ref out_path) = opt.output {
        Box::new(File::create(out_path).unwrap()) as Box<dyn Write>
    } else {
        Box::new(::std::io::stdout()) as Box<dyn Write>
    });

    let mut graph = fbx::Graph::new(opt.fbx_path.clone());

    // Add implicit root node.
    graph.add_node(fbx::Node::new(0));

    fbx::traverse(&mut graph, &mut src);

    if let Some(ref filter_path) = opt.filter {
        let filters: fbx::filter::Filters = {
            use std::io::Read;
            let mut filter_json_str = String::new();
            File::open(filter_path)
                .unwrap()
                .read_to_string(&mut filter_json_str)
                .unwrap();
            serde_json::from_str(&filter_json_str).unwrap()
        };
        filters.apply(&mut graph);
        graph
            .output_visible_nodes(&mut out, filters.show_implicit_nodes.unwrap_or(false))
            .unwrap();
    } else {
        graph.output_all(&mut out).unwrap();
    }
}
