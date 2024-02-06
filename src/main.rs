use cli::{Args, Commands, Format, Parser, ProfileCommands};
use colored::{ColoredString, Colorize};
use futures::StreamExt;
use opendal::EntryMode;
use remote_files::{
    client::StatEntry,
    configuration::{
        self, create_client, Configuration, ConfigurationLayer, Persistence, PersistenceLayer,
        CONFIGURATION_FILEPATH_ENV_VAR,
    },
    error::ClientError,
};
use std::{collections::HashMap, io::Write, path::PathBuf, process};
use thiserror::Error;
use tokio::fs;
use utils::what_next;

use crate::utils::NextAction;

mod cli;
mod utils;

const RF_ICON: &str = "🪣 ";

#[derive(Debug, Error)]
enum CliError {
    #[error("{}", 0)]
    Initialization(String),
    #[error("{}", 0)]
    Configuration(String),
    #[error(transparent)]
    CliError(#[from] ClientError),
}

async fn set_folder() -> Result<(), CliError> {
    let folder = configuration::get_default_folder().unwrap();

    if !fs::try_exists(folder.as_path()).await.map_err(|_| {
        CliError::Initialization(format!("cannot stat file or directory '{:?}'", folder))
    })? {
        fs::create_dir_all(folder.as_path()).await.map_err(|err| {
            CliError::Initialization(format!("cannot create dir {:?}: {}", folder, err))
        })?;
    }

    Ok(())
}

fn get_profile(
    args_profile: Option<String>,
    pers: &Persistence,
    cfg: &Configuration,
) -> Result<String, CliError> {
    // get current profile if any
    let profile = args_profile
        .or_else(|| pers.current.clone())
        .ok_or(CliError::Initialization("no profile selected".to_string()))?;
    let profile_ref = profile.as_str();

    cfg.contains_key(profile_ref)
        .then(|| profile.clone())
        .ok_or(CliError::Initialization(format!(
            "no profile '{profile_ref}' found"
        )))
}

fn list_profiles(profiles: Vec<&String>, current: Option<&str>) {
    utils::log_profiles_table(profiles, current);
    println!();
}

fn list_entries(items: &[StatEntry], should_paginate: bool) {
    utils::log_files_table(items, true, should_paginate);
    println!();
}

enum Level {
    Info,
    Error,
}

impl Level {
    fn as_str(self) -> ColoredString {
        match self {
            Level::Info => "info".green(),
            Level::Error => "error".bold().red(),
        }
    }
}

fn welcome() {
    println!(
        "\n{}{} '{}'\n",
        "Welcome to ".magenta(),
        RF_ICON,
        "remote-files".bold().magenta()
    );
    println!("{}\n", "-".repeat(50))
}

fn ok(text: impl AsRef<str>) {
    println!("[{}]: {}", Level::Info.as_str(), text.as_ref());
}

fn error(text: impl AsRef<str>) {
    println!("[{}]: {}", Level::Error.as_str(), text.as_ref());
}

async fn run() -> Result<(), CliError> {
    set_folder().await?;

    let env_wd = std::env::var(CONFIGURATION_FILEPATH_ENV_VAR)
        .map(|str| PathBuf::from(str))
        .ok();

    let mut cfg_layer = ConfigurationLayer::try_init(env_wd.as_deref())
        .await
        .unwrap();
    let cfg = cfg_layer.get_mut();
    let mut pers_layer = PersistenceLayer::try_init(None).await.unwrap();
    let pers = pers_layer.get_mut();

    let args = Args::parse();

    match args.command {
        Commands::Profile { command } => match command {
            ProfileCommands::List => {
                welcome();

                ok("here's the list of available profiles\n");

                list_profiles(
                    cfg.keys().collect::<Vec<_>>(),
                    get_profile(args.profile, pers, cfg).ok().as_deref(),
                );

                ok("use 'profile set' to change the current profile\n");
            }
            ProfileCommands::Add {
                interactive: _,
                format,
                cfg: next_cfg,
            } => {
                let mut parsed_cfg: Configuration = match format {
                    Format::Json => serde_json::from_str(&next_cfg).unwrap(),
                    Format::Yaml => serde_yaml::from_str(&next_cfg).unwrap(),
                };

                parsed_cfg.drain().for_each(|(key, value)| {
                    cfg.insert(key, value);
                });
                cfg_layer.persist().await.unwrap();
            }
            ProfileCommands::Get => {
                welcome();
                ok(format!(
                    "current profile is {}\n",
                    get_profile(args.profile, pers, cfg)?
                ));
            }
            ProfileCommands::Set { name } => {
                welcome();
                if cfg.contains_key(&name) {
                    pers.current = Some(name.to_string());
                    pers_layer.persist().await.map_err(|err| {
                        CliError::Configuration(format!("cannot persist configuration: {}", err))
                    })?;

                    ok(format!(
                        "current profile set to '{}'\n",
                        name.as_str().bold().green()
                    ));
                } else {
                    return Err(CliError::Configuration(format!(
                        "profile '{name}' does not exist"
                    )));
                }
            }
            ProfileCommands::Dump { format } => {
                if let Some(profile) = pers.current.as_ref() {
                    let cfg = cfg
                        .iter()
                        .filter(|&(name, _)| name == profile)
                        .collect::<HashMap<_, _>>();
                    match format {
                        Format::Json => {
                            println!("{}", serde_json::to_string_pretty(&cfg).unwrap());
                        }
                        Format::Yaml => {
                            println!("{}", serde_yaml::to_string(&cfg).unwrap());
                        }
                    }
                } else {
                    return Err(CliError::Configuration(format!("no profile selected")));
                }
            }
            ProfileCommands::Remove { name } => {
                welcome();
                if cfg.contains_key(&name) {
                    cfg.remove(&name);
                    cfg_layer.persist().await.map_err(|err| {
                        CliError::Configuration(format!("cannot persist configuration: {}", err))
                    })?;

                    if pers.current.as_ref() == Some(&name) {
                        pers.current = None;
                        pers_layer.persist().await.map_err(|err| {
                            CliError::Configuration(format!(
                                "cannot persist configuration: {}",
                                err
                            ))
                        })?;
                    }

                    ok(format!(
                        "removed profile '{}'\n",
                        name.as_str().bold().green()
                    ));
                } else {
                    return Err(CliError::Configuration(format!(
                        "profile '{name}' does not exist"
                    )));
                }
            }
        },
        Commands::List { path, paginate } => {
            welcome();

            let mut path = path.unwrap_or("/".to_string());
            let profile = get_profile(args.profile, pers, cfg)?;
            let path = match path.as_bytes() {
                &[.., b'/'] => path,
                _ => {
                    path.push('/');
                    path
                }
            };

            ok(format!(
                "listing content of folder '{}' for profile '{}'\n",
                path.as_str().bold().green(),
                profile.bold().cyan()
            ));

            let mut page_count = 0;
            let client = create_client(&profile, &cfg)?.unwrap();
            let should_paginate = paginate.is_some();
            let mut stream = client.list(&path, paginate).await?;

            while let Some(page) = stream.next().await {
                page_count += 1;
                ok(format!(
                    "printing {} {}\n",
                    "page".bold().cyan(),
                    &page_count.to_string().bold().cyan()
                ));

                let items = page.await;

                list_entries(items.as_ref(), should_paginate);

                if should_paginate {
                    print!("press 'q' to quit, type an integer to download a file, or anything else to keep scrolling 👀 :");
                    std::io::stdout().flush().unwrap();

                    let item_to_download = match what_next().await {
                        NextAction::Quit => break,
                        NextAction::Next => continue,
                        NextAction::Print(idx) => items.get(idx - 1),
                    };

                    if let Some((name, _, _, mode)) = item_to_download {
                        if mode != &EntryMode::FILE {
                            error("download is available for files only\n");
                        } else {
                            let filepath = PathBuf::from(&path).join(name);
                            let filepath = filepath.to_str().unwrap();
                            let content = client.download(filepath).await?;

                            println!();
                            ok(format!("printing '{filepath}'\n"));

                            std::io::stdout().write_all(&content).unwrap();
                            std::io::stdout().flush().unwrap();

                            println!();
                            ok(format!("=== EOF ===\n"));
                        }

                        break;
                    }
                }

                println!()
            }
        }
        Commands::Delete { path } => {
            welcome();

            let profile = get_profile(args.profile, pers, cfg)?;

            ok(format!(
                "deleting file '{}' for profile '{}'\n",
                path,
                profile.bold().cyan()
            ));

            let client = create_client(&profile, &cfg)?.unwrap();
            client.delete(&path).await?;
        }
        Commands::Upload { src, mut dest } => {
            welcome();

            let profile = get_profile(args.profile, pers, cfg)?;
            let dest = match dest.as_bytes() {
                &[.., b'/'] => dest,
                _ => {
                    dest.push('/');
                    dest
                }
            };

            ok(format!(
                "uploading file '{}' to folder '{}' for profile '{}'\n",
                src,
                dest.as_str().bold().green(),
                profile.bold().cyan()
            ));

            let client = create_client(&profile, &cfg)?.unwrap();

            client.upload(&src, &dest, None).await?;
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error(format!("{:#?}\n", err));
        process::exit(1);
    }
}
