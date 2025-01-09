use std::time::Duration;

use clap::Parser;

use llam::{
    cli::{Config, DiagnosticSetting, DocSetting, Subcommand, LLAM}, frames, logging::{colors, Spinner, Stream}, Error, Manager
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

    let mut manager = Manager::new(
        path,
        Spinner::new(
            Stream::Stdout,
            frames!(
                ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
                Duration::from_millis(80),
                colors::xterm::PaleGoldenrod
            )
        )
    )?;

    match llam.command {
        Subcommand::Add { addons } => manager.add(addons)?,
        Subcommand::Remove(addons) => manager.remove(addons)?,
        Subcommand::Update(addons) => manager.update(addons)?,
        Subcommand::Clean => manager.clean()?,
        Subcommand::List => for (name, addon) in manager.rc.get_addons() {
            println!("  {name}: {:?}", addon.target);
        },
        Subcommand::Config { subcommand } => match subcommand {
            Config::Doc { setting } => match setting {
                DocSetting::Package { patterns } => {
                    match manager.rc.doc.as_mut() {
                        Some(d) => d.package_name.extend(patterns),
                        None => {
                            manager.rc.doc = Some(llam::lua_rc::Doc {
                                package_name: patterns.into_iter().collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.rc.write()?;
                }
                DocSetting::Private { patterns } => {
                    match manager.rc.doc.as_mut() {
                        Some(d) => d.private_name.extend(patterns),
                        None => {
                            manager.rc.doc = Some(llam::lua_rc::Doc {
                                private_name: patterns.into_iter().collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.rc.write()?;
                }
                DocSetting::Protected { patterns } => {
                    match manager.rc.doc.as_mut() {
                        Some(d) => d.protected_name.extend(patterns),
                        None => {
                            manager.rc.doc = Some(llam::lua_rc::Doc {
                                protected_name: patterns.into_iter().collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.rc.write()?;
                }
            },
            Config::Diagnostic { setting } => match setting {
                DiagnosticSetting::Disable { diagnostics } => {
                    match manager.rc.diagnostics.as_mut() {
                        Some(d) => d.disable.extend(diagnostics),
                        None => {
                            manager.rc.diagnostics = Some(llam::lua_rc::Diagnostics {
                                disable: diagnostics,
                                ..Default::default()
                            })
                        }
                    }
                    manager.rc.write()?;
                }
                DiagnosticSetting::Enable { diagnostics } => {
                    if let Some(d) = manager.rc.diagnostics.as_mut() {
                        d.disable.retain(|item| !diagnostics.contains(item));
                        manager.rc.write()?;
                    }
                }
                DiagnosticSetting::AddGlobal { globals } => {
                    match manager.rc.diagnostics.as_mut() {
                        Some(d) => d.globals.extend(globals),
                        None => {
                            manager.rc.diagnostics = Some(llam::lua_rc::Diagnostics {
                                globals,
                                ..Default::default()
                            })
                        }
                    }
                    manager.rc.write()?;
                }
                DiagnosticSetting::RemoveGlobal { globals } => {
                    if let Some(d) = manager.rc.diagnostics.as_mut() {
                        d.globals.retain(|item| !globals.contains(item));
                        manager.rc.write()?;
                    }
                }
                DiagnosticSetting::Severity { severity } => {
                    match manager.rc.diagnostics.as_mut() {
                        Some(d) => d
                            .severity
                            .extend(severity.into_iter().map(|s| (s.key, s.value))),
                        None => {
                            manager.rc.diagnostics = Some(llam::lua_rc::Diagnostics {
                                severity: severity.into_iter().map(|s| (s.key, s.value)).collect(),
                                ..Default::default()
                            })
                        }
                    }
                    manager.rc.write()?;
                }
            },
        },
    }

    Ok(())
}
