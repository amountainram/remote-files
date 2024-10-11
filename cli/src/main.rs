use anyhow::{anyhow, Context as _, Result};
use colored::Colorize;
use inquire::{Confirm, Select};
use remote_files::{
    cli::{Args, Commands, Parser as _, ProfileCommands},
    configuration,
    util::{self, ProfileInfo},
};
use std::{collections::HashSet, process};
// use cli::{Args, Commands, Parser, ProfileCommands};
// use client::StatEntry;
// use colored::{ColoredString, Colorize};
// use futures::StreamExt;
// use opendal::EntryMode;
// //configuration::{
// //    self, create_client, Configuration, ConfigurationLayer, Persistence, PersistenceLayer,
// //    CONFIGURATION_FILEPATH_ENV_VAR,
// //},
// use error::Client;
// use remote_files::{
//     configuration,
//     util::{log_files_table, log_profiles_table, what_next, NextAction},
// };
// use std::{collections::HashMap, io::Write, path::PathBuf, process};
// use thiserror::Error;
// use tokio::fs;

// mod buckets;
// mod cli;
// mod client;
// mod error;
// mod util;

// const RF_ICON: &str = "🪣 ";

// #[derive(Debug, Error)]
// enum CliError {
//     #[error("{}", 0)]
//     Initialization(String),
//     #[error("{}", 0)]
//     Configuration(String),
//     #[error(transparent)]
//     Cli(#[from] Client),
// }

// async fn set_folder() -> Result<(), CliError> {
//     // let home = get_home_folder()
//     //     .await
//     //     .with_context(|| "retrieving home folder location")?;
//     let folder = configuration::get_default_folder().unwrap();

//     if !fs::try_exists(folder.as_path()).await.map_err(|_| {
//         CliError::Initialization(format!("cannot stat file or directory '{:?}'", folder))
//     })? {
//         fs::create_dir_all(folder.as_path()).await.map_err(|err| {
//             CliError::Initialization(format!("cannot create dir {:?}: {}", folder, err))
//         })?;
//     }

//     Ok(())
// }

// fn get_profile(
//     args_profile: Option<String>,
//     pers: &Persistence,
//     cfg: &Configuration,
// ) -> Result<String, CliError> {
//     // get current profile if any
//     let profile = args_profile
//         .or_else(|| pers.current.clone())
//         .ok_or(CliError::Initialization("no profile selected".to_string()))?;
//     let profile_ref = profile.as_str();

//     cfg.contains_key(profile_ref)
//         .then(|| profile.clone())
//         .ok_or(CliError::Initialization(format!(
//             "no profile '{profile_ref}' found"
//         )))
// }

// fn list_profiles(profiles: Vec<&String>, current: Option<&str>) {
//     log_profiles_table(profiles, current);
//     println!();
// }

// fn list_entries(items: &[StatEntry], should_paginate: bool) {
//     log_files_table(items, true, should_paginate);
//     println!();
// }

// enum Level {
//     Info,
//     Error,
// }

// impl Level {
//     fn into_str(self) -> ColoredString {
//         match self {
//             Level::Info => "info".green(),
//             Level::Error => "error".bold().red(),
//         }
//     }
// }

// fn welcome() {
//     println!(
//         "\n{}{} '{}'\n",
//         "Welcome to ".magenta(),
//         RF_ICON,
//         "remote-files".bold().magenta()
//     );
//     println!("{}\n", "-".repeat(50))
// }

// fn ok(text: impl AsRef<str>) {
//     println!("[{}]: {}", Level::Info.into_str(), text.as_ref());
// }

// fn error(text: impl AsRef<str>) {
//     println!("[{}]: {}", Level::Error.into_str(), text.as_ref());
// }

async fn run() -> Result<()> {
    let Args { profile, command } = Args::parse();

    let home = configuration::get_home_folder()
        .await
        .with_context(|| "retrieving home folder location")?;
    let (mut cli_state, mut cfg) = configuration::try_init(&home).await?;

    util::welcome();

    match command {
        Commands::Profile { command } => match command {
            ProfileCommands::List => {
                let profiles = cfg
                    .get()
                    .buckets
                    .keys()
                    .map(|name| ProfileInfo {
                        name,
                        current: cli_state
                            .get()
                            .current
                            .as_ref()
                            .map(|p| p == name)
                            .unwrap_or_default(),
                    })
                    .collect::<Vec<_>>();
                if profiles.is_empty() {
                    util::msg_ok(format!("{} available profiles", "no".bold()));
                } else {
                    util::msg_ok("here's the list of available profiles");
                    util::print_table(&util::list_profiles(&profiles));
                    util::msg_ok("use 'profile set' to change the current profile");
                }
            }
            // ProfileCommands::Add {
            //     interactive: _,
            //     cfg: next_cfg,
            // } => {
            //     let mut parsed_cfg: Configuration = serde_json::from_str(&next_cfg).unwrap();

            //     parsed_cfg.drain().for_each(|(key, value)| {
            //         cfg.insert(key, value);
            //     });
            //     cfg_layer.persist().await.unwrap();
            // }
            ProfileCommands::Get => {
                util::msg_ok(format!(
                    "current profile is {}",
                    util::get_profile(
                        profile.as_deref(),
                        cli_state.get().current.as_deref(),
                        cfg.get()
                            .buckets
                            .keys()
                            .map(|k| k.as_str())
                            .collect::<HashSet<_>>()
                    )?
                    .bold()
                ));
            }
            ProfileCommands::Set { name } => {
                let opts = &cfg.get().buckets;
                let state = cli_state.get_mut();

                let current = if let Some(name) = name {
                    opts.contains_key(&name)
                        .then_some(name.clone())
                        .ok_or(anyhow!("profile '{name}' does not exist"))
                } else {
                    let current =
                        Select::new("select the next current profile", opts.keys().collect())
                            .prompt()
                            .cloned()
                            .with_context(|| "while selecting next current profile");
                    util::msg_ok("");
                    current
                }?;

                state.current = Some(current.clone());
                cli_state
                    .persist()
                    .await
                    .with_context(|| "persisting configuration")?;

                util::msg_ok(format!(
                    "current profile set to '{}'",
                    current.as_str().bold().green()
                ));
            }
            // ProfileCommands::Dump => {
            //     if let Some(profile) = pers.current.as_ref() {
            //         let cfg = cfg
            //             .iter()
            //             .filter(|&(name, _)| name == profile)
            //             .collect::<HashMap<_, _>>();
            //         println!("{}", serde_json::to_string_pretty(&cfg).unwrap());
            //     } else {
            //         return Err(CliError::Configuration("no profile selected".to_string()));
            //     }
            // }
            ProfileCommands::Remove { name, confirm } => {
                let opts = &cfg.get().buckets;

                let (name, confirm) = if let Some(name) = name {
                    cfg.get()
                        .buckets
                        .contains_key(&name)
                        .then_some((name.clone(), confirm))
                        .ok_or(anyhow!("profile '{name}' does not exist"))
                } else {
                    let current =
                        Select::new("select the profile to remove", opts.keys().collect())
                            .prompt()
                            .cloned()
                            .map(|name| (name, false))
                            .with_context(|| "while selecting a profile");
                    util::msg_ok("");
                    current
                }?;

                let confirm = if !confirm {
                    let confirm =
                        Confirm::new(&format!("Are you sure to delete profile '{name}'?"))
                            .with_default(false)
                            .prompt()
                            .with_context(|| "while confirming profile deletion")?;
                    util::msg_ok("");
                    confirm
                } else {
                    true
                };

                if confirm {
                    cfg.get_mut().buckets.remove(&name);
                    let current = &mut cli_state.get_mut().current;
                    if current.as_deref() == Some(&name) {
                        *current = Some(name.clone());
                    }

                    let (cfg_task, cli_state_task) =
                        tokio::join!(cfg.persist(), cli_state.persist());
                    cfg_task.with_context(|| "persisting configuration")?;
                    cli_state_task.with_context(|| "persisting cli state")?;
                    util::msg_ok(format!(
                        "removed profile '{}'",
                        name.as_str().bold().green()
                    ));
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(), //     Commands::List { path, paginate } => {
                               //         welcome();

                               //         let mut path = path.unwrap_or("/".to_string());
                               //         let profile = get_profile(args.profile, pers, cfg)?;
                               //         let path = match path.as_bytes() {
                               //             &[.., b'/'] => path,
                               //             _ => {
                               //                 path.push('/');
                               //                 path
                               //             }
                               //         };

                               //         ok(format!(
                               //             "listing content of folder '{}' for profile '{}'\n",
                               //             path.as_str().bold().green(),
                               //             profile.bold().cyan()
                               //         ));

                               //         let mut page_count = 0;
                               //         let client = create_client(&profile, cfg)?.unwrap();
                               //         let should_paginate = paginate.is_some();
                               //         let mut stream = client.list(&path, paginate).await?;

                               //         while let Some(page) = stream.next().await {
                               //             page_count += 1;
                               //             ok(format!(
                               //                 "printing {} {}\n",
                               //                 "page".bold().cyan(),
                               //                 &page_count.to_string().bold().cyan()
                               //             ));

                               //             let items = page.await;

                               //             list_entries(items.as_ref(), should_paginate);

                               //             if should_paginate {
                               //                 print!("press 'q' to quit, type an integer to download a file, or anything else to keep scrolling 👀 :");
                               //                 std::io::stdout().flush().unwrap();

                               //                 let item_to_download = match what_next().await {
                               //                     NextAction::Quit => break,
                               //                     NextAction::Next => continue,
                               //                     NextAction::Print(idx) => items.get(idx - 1),
                               //                 };

                               //                 if let Some((name, _, _, mode)) = item_to_download {
                               //                     if mode != &EntryMode::FILE {
                               //                         error("download is available for files only\n");
                               //                     } else {
                               //                         let filepath = PathBuf::from(&path).join(name);
                               //                         let filepath = filepath.to_str().unwrap();
                               //                         let content = client.download(filepath).await?;

                               //                         println!();
                               //                         ok(format!("printing '{filepath}'\n"));

                               //                         std::io::stdout().write_all(&content).unwrap();
                               //                         std::io::stdout().flush().unwrap();

                               //                         println!();
                               //                         ok("=== EOF ===\n");
                               //                     }

                               //                     break;
                               //                 }
                               //             }

                               //             println!()
                               //         }
                               //     }
                               //     Commands::Delete { path } => {
                               //         welcome();

                               //         let profile = get_profile(args.profile, pers, cfg)?;

                               //         ok(format!(
                               //             "deleting file '{}' for profile '{}'\n",
                               //             path,
                               //             profile.bold().cyan()
                               //         ));

                               //         let client = create_client(&profile, cfg)?.unwrap();
                               //         client.delete(&path).await?;
                               //     }
                               //     Commands::Upload { src, mut dest } => {
                               //         welcome();

                               //         let profile = get_profile(args.profile, pers, cfg)?;
                               //         let dest = match dest.as_bytes() {
                               //             &[.., b'/'] => dest,
                               //             _ => {
                               //                 dest.push('/');
                               //                 dest
                               //             }
                               //         };

                               //         ok(format!(
                               //             "uploading file '{}' to folder '{}' for profile '{}'\n",
                               //             src,
                               //             dest.as_str().bold().green(),
                               //             profile.bold().cyan()
                               //         ));

                               //         let client = create_client(&profile, cfg)?.unwrap();

                               //         client.upload(&src, &dest, None).await?;
                               //     }
    };

    util::msg_ok("");

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err:#?}");
        process::exit(1);
    }
}
