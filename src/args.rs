use std::ffi::OsString;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(name = "Dir Kill", version, author, about)]
pub struct DirKillArgs {
    #[clap(
        short,
        long,
        default_value = "node_modules",
        help = "The directory to remove"
    )]
    pub target: OsString,

    #[clap(short, long, default_value = ".", help = "The directory to search")]
    pub dir: OsString,
}