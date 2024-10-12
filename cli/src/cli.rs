pub use clap::Parser;
use clap::Subcommand;
use configuration::url_path::UrlPath;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Lists available profiles
    #[clap(aliases = &["l", "li"])]
    List,
    #[clap(aliases = &["a"])]
    Add {
        /// enables add in interactive mode
        #[arg(short, default_value_t = false)]
        interactive: bool,

        /// enables add in interactive mode
        #[arg(value_name = "CONFIG")]
        cfg: String,
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
    /// Dumps current profile configuration
    #[clap(aliases = &["d"])]
    Dump,
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
        path: Option<UrlPath>,
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
    Upload { src: String, dest: String },
    #[clap(aliases = &["dw", "down"])]
    Download { src: String, dest: String },
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
