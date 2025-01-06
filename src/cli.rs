use std::{path::PathBuf, str::FromStr};

use crate::{lua_rc::{diagnostics::Diagnostic, Severity}, manager::SomeOrAll, Addon};

/// Lua Language Addon Manager
///
/// Used to install and manage lua language server addons. The idea being that it installs them to a set location
/// then adds a `.luarc.json` file to the current location to expose the addons.
#[derive(Debug, clap::Parser)]
#[command(name = "llam", version, about, long_about = None)]
pub struct LLAM {
    /// Manually define the root path of the project
    #[arg(long)]
    pub path: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Subcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Add one or more lua language addons
    Add { addons: Vec<Addon> },
    /// Remove one or more lua language addons
    Remove(ListOrAll),
    /// Update one, many, or all lua language addons
    Update(ListOrAll),
    /// Remove any addons that are not in the config/lockfile
    Clean,
    /// Update the .luarc.json config settings
    Config {
        #[command(subcommand)]
        subcommand: Config,
    },
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct ListOrAll {
    pub addons: Vec<Addon>,
    #[arg(long)]
    pub all: bool,
}

impl From<ListOrAll> for SomeOrAll<Addon> {
    fn from(value: ListOrAll) -> Self {
       if value.all {
           SomeOrAll::All
       } else {
           SomeOrAll::Some(value.addons)
       }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum Config {
    /// Change a diagnostic setting
    Diagnostic {
        #[command(subcommand)]
        setting: DiagnosticSetting,
    },
    /// Change settings for setting table keys to package private, private, or protected
    Doc {
        #[command(subcommand)]
        setting: DocSetting,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum DiagnosticSetting {
    /// Disable a diagnostic
    Disable { diagnostics: Vec<Diagnostic> },
    /// Enable a diagnostic that has been disabled
    Enable { diagnostics: Vec<Diagnostic> },
    /// Add variables that are declared as globals
    AddGlobal { globals: Vec<String> },
    /// Remove variables that are declared as globals
    RemoveGlobal { globals: Vec<String> },
    /// Set the severity of diagnostics
    Severity {
        severity: Vec<Set<Diagnostic, Severity>>,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum DocSetting {
    /// Set patterns to mark table keys as package private
    Package { patterns: Vec<String> },
    /// Set patterns to mark table keys as private
    Private { patterns: Vec<String> },
    /// Set patterns to mark table keys as protected
    Protected { patterns: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct Set<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> FromStr for Set<K, V>
where
    K: FromStr,
    K::Err: ToString,
    V: FromStr,
    V::Err: ToString,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains("=") {
            return Err("invalid set value, expected [key]=[value]".to_string());
        }

        let (key, value) = s.split_once('=').unwrap();

        Ok(Self {
            key: K::from_str(key).map_err(|e| e.to_string())?,
            value: V::from_str(value).map_err(|e| e.to_string())?,
        })
    }
}
