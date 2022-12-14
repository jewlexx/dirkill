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

    #[clap(
        long,
        help = "The highlight color to use for the selected entry. Must be a hex value"
    )]
    pub color: Option<String>,

    #[clap(short = 'l', long, help = "Whether or not to follow symlinks")]
    pub follow_links: bool,
}
