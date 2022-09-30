use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
struct DirKillArgs {}

fn main() {
    let args = DirKillArgs::parse();

    println!("Hello, world!");
}
