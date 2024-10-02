use clap::Parser;

use llam::{
    cli::{Config, DiagnosticSetting, DocSetting, ListOrAll, Subcommand, LLAM},
    Error, Manager,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let llam = LLAM::parse();

    let path = llam.path.unwrap_or(std::env::current_dir()?);
    if !path.exists() {
        return Err(Error::custom(format!(
            "the project path does not exist: {path:?}"
        )));
    }

    let mut manager = Manager::new(path)?;

    match llam.command {
        Subcommand::Add { addons } => manager.add(addons)?,
        Subcommand::Remove(ListOrAll { addons, all }) => manager.remove(addons, all)?,
        Subcommand::Update(ListOrAll { addons, all }) => manager.update(addons, all)?,
        Subcommand::Clean => manager.clean()?,
        Subcommand::Config { subcommand } => match subcommand {
            Config::Doc { setting } => match setting {
                DocSetting::Package { patterns } => {
                    match manager.config.doc.as_mut() {
                        Some(d) => d.package_name.extend(patterns),
                        None => {
                            manager.config.doc = Some(llam::config::Doc {
                                package_name: patterns.into_iter().collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.config.write()?;
                }
                DocSetting::Private { patterns } => {
                    match manager.config.doc.as_mut() {
                        Some(d) => d.private_name.extend(patterns),
                        None => {
                            manager.config.doc = Some(llam::config::Doc {
                                private_name: patterns.into_iter().collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.config.write()?;
                }
                DocSetting::Protected { patterns } => {
                    match manager.config.doc.as_mut() {
                        Some(d) => d.protected_name.extend(patterns),
                        None => {
                            manager.config.doc = Some(llam::config::Doc {
                                protected_name: patterns.into_iter().collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.config.write()?;
                }
            },
            Config::Diagnostic { setting } => match setting {
                DiagnosticSetting::Disable { diagnostics } => {
                    match manager.config.diagnostics.as_mut() {
                        Some(d) => d.disable.extend(diagnostics),
                        None => {
                            manager.config.diagnostics = Some(llam::config::Diagnostics {
                                disable: diagnostics,
                                ..Default::default()
                            })
                        }
                    }
                    manager.config.write()?;
                }
                DiagnosticSetting::Enable { diagnostics } => {
                    if let Some(d) = manager.config.diagnostics.as_mut() {
                        d.disable = d
                            .disable
                            .drain(..)
                            .filter(|item| !diagnostics.contains(item))
                            .collect();
                        manager.config.write()?;
                    }
                }
                DiagnosticSetting::AddGlobal { globals } => {
                    match manager.config.diagnostics.as_mut() {
                        Some(d) => d.globals.extend(globals),
                        None => {
                            manager.config.diagnostics = Some(llam::config::Diagnostics {
                                globals,
                                ..Default::default()
                            })
                        }
                    }
                    manager.config.write()?;
                }
                DiagnosticSetting::RemoveGlobal { globals } => {
                    if let Some(d) = manager.config.diagnostics.as_mut() {
                        d.globals = d
                            .globals
                            .drain(..)
                            .filter(|item| !globals.contains(item))
                            .collect();
                        manager.config.write()?;
                    }
                }
                DiagnosticSetting::Severity { severity } => {
                    match manager.config.diagnostics.as_mut() {
                        Some(d) => d
                            .severity
                            .extend(severity.into_iter().map(|s| (s.key, s.value))),
                        None => {
                            manager.config.diagnostics = Some(llam::config::Diagnostics {
                                severity: severity.into_iter().map(|s| (s.key, s.value)).collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.config.write()?;
                }
            },
        },
    }

    Ok(())
}
