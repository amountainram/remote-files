use remote_files_configuration::{CliState, Configuration};
use std::{fs, io, path::Path};

fn build_configuration_schema() -> io::Result<()> {
    println!("cargo:rerun-if-changed=../configuration/src");

    let path = Path::new("schemas");

    fs::create_dir_all(path)?;

    // cli configuration
    let configuration_schema = schemars::schema_for!(Configuration);
    fs::write(
        path.join("configuration.schema.json"),
        serde_json::to_string_pretty(&configuration_schema)?,
    )?;

    // cli persisted state
    let cli_state_schema = schemars::schema_for!(CliState);
    fs::write(
        path.join("cli_state.schema.json"),
        serde_json::to_string_pretty(&cli_state_schema)?,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_configuration_schema()?;

    Ok(())
}
