pub use clap::Parser;
use clap::Subcommand;

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
    Set { name: String },
    /// Sets current profile
    #[clap(aliases = &["r", "rm"])]
    Remove { name: String },
    /// Dumps current profile configuration
    #[clap(aliases = &["d"])]
    Dump,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Access to available profiles
    #[clap(aliases = &["p", "pr", "prof"])]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    #[clap(aliases = &["l", "li"])]
    List {
        path: Option<String>,
        #[arg(short, long)]
        paginate: Option<usize>,
    },
    #[clap(aliases = &["d", "del"])]
    Delete { path: String },
    #[clap(aliases = &["u", "up"])]
    Upload { src: String, dest: String },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// override current profile if any
    #[arg(short, long)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}
