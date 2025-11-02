use clap::Parser;
use std::error::Error;

use std::path::Path;
use std::{
    fs::{self, File},
    io::Read,
};

use mzcache2::{error::MzResult, file::parse_cachefile, index::read_index_file};
use mzcache2::index::{self, Hash};

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

/*
*   simple util to extract the cache to files
*
*/
fn main() {
    let args = Cli::parse();

    println!("{:?}",args.path);

    let (index, entries) = mzcache2::parse_cache_folder(&args.path).unwrap();

    for r in index.records {
        println!("{r:?}");
    }

    for (hash, e) in entries{
        println!("{hash}: {e}");
    }

}
