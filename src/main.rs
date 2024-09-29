use std::path::PathBuf;

use clap::Parser;

use llam::{config::LuaRc, Addon, Error, Manager};

/// Lua Language Addon Manager
/// 
/// Used to install and manage lua language server addons. The idea being that it installs them to a set location
/// then adds a `.luarc.json` file to the current location to expose the addons.
#[derive(Debug, Parser)]
#[command(name = "llam", version, about, long_about = None)]
struct LLAM {
    /// Manually define the root path of the project
    #[arg(long)]
    path: Option<PathBuf>,
    #[command(subcommand)]
    command: Subcommand,     
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    /// Add one or more lua language addons
    Add {
        addons: Vec<Addon>
    },
    /// Remove one or more lua language addons
    Remove(ListOrAll),
    /// Update one, many, or all lua language addons
    Update(ListOrAll),
    /// Remove any addons that are not in the config/lockfile
    Clean,
    Pass,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct ListOrAll {
    addons: Vec<Addon>,
    #[arg(long)]
    all: bool
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let llam = LLAM::parse();
    let mut manager = Manager::new(llam.path.unwrap_or(std::env::current_dir()?))?;

    match llam.command {
        Subcommand::Add { addons } => manager.add(addons)?,
        Subcommand::Remove(ListOrAll { addons, all }) => manager.remove(addons, all)?,
        Subcommand::Update(ListOrAll { addons, all }) => manager.update(addons, all)?,
        Subcommand::Clean => manager.clean()?,
        Subcommand::Pass => {
            let content = std::fs::read("sample.json")?;
            let de = &mut serde_json::Deserializer::from_slice(&content);
            let config: LuaRc = serde_path_to_error::deserialize(de)?;

            std::fs::write("sample-output.json", serde_json::to_string_pretty(&config)?)?;
            //println!("{config:#?}");
        }
    }

    Ok(())
}
