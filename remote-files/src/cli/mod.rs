pub use bucket_inquire::bucket_inquire;
pub use clap::Parser;
use clap::Subcommand;
use remote_files_configuration::{
    url_path::{UrlDirPath, UrlPath},
    Bucket,
};
use std::path::PathBuf;

mod bucket_inquire;

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Lists available profiles
    #[clap(aliases = &["l", "li"])]
    List,
    #[clap(aliases = &["a"])]
    Add {
        /// after insertion set as default
        #[arg(long, default_value_t = false)]
        current: bool,

        /// the name of the configuration to add
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// the plain json configuration of the profile to add.
        /// If none is passed then the cli will fallback to
        /// interactive mode
        #[arg(value_name = "CONFIG")]
        config: Option<Bucket>,
    },
    /// Prints current profile
    #[clap(aliases = &["g"])]
    Get,
    /// Sets current profile
    #[clap(aliases = &["s"])]
    Set { name: Option<String> },
    /// Sets current profile
    #[clap(aliases = &["r", "rm"])]
    Remove {
        name: Option<String>,
        #[arg(short = 'y', default_value_t = false)]
        confirm: bool,
    },
    /// Info about current profile configuration
    #[clap(aliases = &["i"])]
    Info { name: String },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage available profiles (bucket connections)
    #[clap(aliases = &["p", "pr", "prof"])]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Lists files at the given folder path.
    /// A trailing slash will be forced at the end to ensure
    /// only remote directories are listed
    #[clap(aliases = &["l", "li"])]
    List {
        path: Option<UrlDirPath>,
        #[arg(short, long)]
        paginate: Option<usize>,
    },
    /// Deletes a file or a folder at the given path
    #[clap(aliases = &["d", "del"])]
    Delete { path: UrlPath },
    /// Uploads a file from source to destination.
    /// Relative paths are trimmed and the only thing that matters
    /// is the filename which is stripped from the source and
    /// appended to the destination
    #[clap(aliases = &["u", "up"])]
    Upload { src: PathBuf, dest: UrlPath },
    /// Downloads a file from remote to source to local
    /// folder destination
    #[clap(aliases = &["dw", "down"])]
    Download { src: UrlPath, dest: UrlDirPath },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// override current profile
    #[arg(short, long)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}
