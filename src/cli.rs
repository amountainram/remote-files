pub use clap::Parser;
use clap::{Subcommand, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    Json,
    Yaml,
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Lists available profiles
    List,
    Add {
        /// enables add in interactive mode
        #[arg(short, default_value_t = false)]
        interactive: bool,

        /// output format
        #[arg(short, long, value_enum, default_value_t = Format::Json)]
        format: Format,

        /// enables add in interactive mode
        #[arg(value_name = "CONFIG")]
        cfg: String,
    },
    /// Prints current profile
    Get,
    /// Sets current profile
    Set { name: String },
    /// Sets current profile
    Remove { name: String },
    /// Dumps current profile configuration
    Dump {
        /// output format
        #[arg(short, long, value_enum, default_value_t = Format::Json)]
        format: Format,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Access to available profiles
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    List {
        path: String,
        #[arg(short, long)]
        paginate: Option<usize>,
    },
    Delete {
        path: String,
    },
    Upload {
        src: String,
        dest: String,
    },
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
