use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
pub struct DirKillArgs {
    #[clap(
        short,
        long,
        default_value = "node_modules",
        help = "The directory to remove"
    )]
    target: String,
}
