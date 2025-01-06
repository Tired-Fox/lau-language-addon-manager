use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::{
    git::{Cli, ResetType}, logging::{Logger, OrLog, Spinner}, lua_rc::{LuaRc, Workspace}, Addon, Error, ADDONS_DIR
};

pub enum SomeOrAll<S> {
    Some(Vec<S>),
    All
}
impl<S> From<bool> for SomeOrAll<S> {
    fn from(value: bool) -> Self {
        if value {
            SomeOrAll::All
        } else {
            SomeOrAll::Some(Vec::new())
        }
    }
}
impl<S> From<Vec<S>> for SomeOrAll<S> {
    fn from(value: Vec<S>) -> Self {
        SomeOrAll::Some(value)
    }
}

#[derive(Debug)]
pub struct Manager<L: Logger = Spinner> {
    pub base: PathBuf,
    pub config: LuaRc,

    pub logger: L
}

impl<L: Logger> Manager<L> {
    pub fn new(dir: impl AsRef<Path>, logger: L) -> Result<Self, Error> {
        let path = dir.as_ref();
        Ok(Self {
            config: LuaRc::detect(path)?,
            base: path.to_path_buf(),

            logger,
        })
    }

    pub fn clone_addon(&mut self, name: Cow<'static, str>) -> Result<(), Error> {
        // PERF: Return error or log when addon is not in lock file
        if let Some(addon) = self.config.get_addons().get(&name) {
            let temp_name = addon
                .checksum
                .clone()
                .unwrap_or(uuid::Uuid::now_v7().to_string());
            let from = std::env::temp_dir().join(&temp_name);
            let to = self.base.join(ADDONS_DIR).join(addon.name().as_ref());

            if let Err(err) = Cli::clone(std::env::temp_dir(), addon.clone_url(), &temp_name) {
                if from.exists() {
                    std::fs::remove_dir_all(&from)?;
                }
                return Err(err);
            }

            if to.exists() {
                std::fs::remove_dir_all(&to)?;
            }

            if let Some(parent) = to.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::rename(from, to)?;
        }

        Ok(())
    }

    pub fn add(&mut self, addons: impl IntoIterator<Item=Addon>) -> Result<(), Error> {
        let addons = addons.into_iter().collect::<Vec<_>>();
        let total = addons.len().to_string();
        let mut success = 0;

        let addon_path = self.base.join(ADDONS_DIR);
        for addon in addons.iter() {
            let name = addon.name();
            let path = addon_path.join(name.as_ref());
            self.logger.update(format!(
                "{:0>width$}/{total} Cloning {name}",
                success,
                width = total.len()
            ));

            if !path.exists() || !self.config.get_addons().contains_key(name.as_ref()) {
                self.config.update_addon(addon);
                if self.clone_addon(name.clone()).is_err() {
                    self.logger.error(format!("failed to clone addon: {name}"));
                    continue;
                }

                self.logger.success(format!("{name} added"));
            } else {
                let branch_diff = addon
                    .branch
                    .as_ref()
                    .map(|v| Cli::branch_name(&path).map(|n| &n != v).unwrap_or_default())
                    .unwrap_or_default();
                let checksum_diff = addon
                    .checksum
                    .as_ref()
                    .map(|v| {
                        Cli::checksum(&path, None)
                            .map(|n| &n != v)
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();

                self.config.update_addon(addon);
                if branch_diff || checksum_diff {
                    self.logger.warning(format!("{name} update available"));
                }
            };

            success += 1;
        }

        self.logger.update("Updating .luarc.json");

        let path = ADDONS_DIR.to_string();
        match self.config.workspace.as_mut() {
            Some(workspace) => {
                if !workspace.user_third_party.contains(&path) {
                    workspace.user_third_party.push(path);
                }
            }
            None => {
                self.config.workspace = Some(Workspace {
                    user_third_party: Vec::from([path]),
                    ..Default::default()
                });
            }
        }

        if self.config.write().is_err() {
            self.logger.error("failed to write updates to .luarc.json");
        }

        self.logger.success(format!("[Add] {success}/{total} Finished!"));
        Ok(())
    }

    pub fn remove(&mut self, addons: impl Into<SomeOrAll<Addon>>) -> Result<(), Error> {
        let addons = match addons.into() {
            SomeOrAll::Some(addons) => addons,
            SomeOrAll::All => self.config.get_addons().values().cloned().collect()
        };

        let total = addons.len().to_string();
        self.logger.update(format!("{:0>width$}/{total} Removing ...", 0, width = total.len()));

        let addon_path = self.base.join(ADDONS_DIR);
        for (i, addon) in addons.iter().enumerate() {
            let name = addon.name();
            let path = addon_path.join(name.as_ref());
            self.logger.update(format!(
                "{:0>width$}/{total} Removing {name}",
                i + 1,
                width = total.len()
            ));

            if self.config.get_addons().contains_key(name.as_ref()) {
                self.config.get_addons_mut().remove(name.as_ref());
            }

            if path.exists() {
                std::fs::remove_dir_all(path)?;
            }
        }

        if self.config.write().is_err() {
            self.logger.error("failed to write updates to .luarc.json");
        }

        self.logger.success(format!("[Remove] {total}/{total} Finished!"));
        Ok(())
    }

    pub fn update(&mut self, addons: impl Into<SomeOrAll<Addon>>) -> Result<(), Error> {
        // Collect all that are in the config
        let addons = match addons.into() {
            SomeOrAll::Some(addons) => addons,
            SomeOrAll::All => self.config.get_addons().values().cloned().collect()
        };

        let mut success = 0;
        let addon_path = self.base.join(ADDONS_DIR);
        for addon in addons.iter() {
            let name = addon.name();

            if !self.config.get_addons().contains_key(name.as_ref()) {
                continue;
            }
            self.config.update_addon(addon);
            let addon = self.config.get_addons().get(&name).unwrap();

            let path = addon_path.join(name.as_ref());

            self.logger.update(format!("[{name}] Getting branch name"));
            let branch = Cli::branch_name(&path)?;

            self.logger.update(format!("[{name}] Getting default branch name"));
            let default_branch = Cli::default_branch_name(&path)?;

            self.logger.update(format!("[{name}] Getting current checksum"));
            let checksum = Cli::checksum(&path, None)?;

            match addon.branch.as_ref() {
                Some(b) if b != &branch => {
                    self.logger.update(format!("[{name}] Fetching latest repository changes"));
                    if Cli::fetch(&path).is_err() {
                        self.logger.error(format!("[{name}] failed to fetch latest changes from git"));
                        continue;
                    };

                    self.logger.update(format!("[{name}] Switching to branch `{b}`"));
                    if Cli::switch(&path, b).is_err() {
                        self.logger.error(format!("[{name}] failed to switch git branches"));
                        continue;
                    };

                    self.logger.update(format!("[{name}] Pulling latest changes"));
                    if Cli::pull(&path, false).is_err() {
                        self.logger.error(format!("[{name}] failed to pull latest changes"));
                        continue;
                    };

                    if let Some(checksum) = addon.checksum.as_deref() {
                        self.logger.update(format!(
                            "[{name}] Setting branch to checksum `{checksum}`"
                        ));
                        if Cli::reset(&path, ResetType::Hard, Some(checksum)).is_err() {
                            self.logger.error(format!("[{name}] failed to reset git branch"));
                            continue;
                        };
                    }
                }
                None if branch != default_branch => {
                    self.logger.update(format!("[{name}] Fetching latest repository changes"));
                    if Cli::fetch(&path).is_err() {
                        self.logger.error(format!("[{name}] failed to fetch latest changes from git"));
                        continue;
                    };

                    self.logger.update(format!("[{name}] Switching to branch `{default_branch}`"));
                    if Cli::switch(&path, &default_branch).is_err() {
                        self.logger.error(format!("[{name}] failed to switch git branches"));
                        continue;
                    };

                    self.logger.update(format!("[{name}] Pulling latest changes"));
                    if Cli::pull(&path, false).is_err() {
                        self.logger.error(format!("[{name}] failed to pull latest changes"));
                        continue;
                    };

                    if let Some(checksum) = addon.checksum.as_deref() {
                        self.logger.update(format!(
                            "[{name}] Setting branch to checksum `{checksum}`"
                        ));
                        if Cli::reset(&path, ResetType::Hard, Some(checksum)).is_err() {
                            self.logger.error(format!("[{name}] failed to set git branch"));
                            continue;
                        };
                    }
                }
                _ => match addon.checksum.as_ref() {
                    Some(c) if c != &checksum => {
                        self.logger.update(format!("[{name}] Fetching latest repository changes"));
                        if Cli::fetch(&path).is_err() {
                            self.logger.error(format!("[{name}] failed to fetch latest changes from git"));
                            continue;
                        };
                        self.logger.update(format!("[{name}] Setting branch to checksum `{c}`"));
                        if Cli::reset(&path, ResetType::Hard, Some(c)).is_err() {
                            self.logger.error(format!("[{name}] failed to set git branch"));
                            continue;
                        };
                    }
                    None => {
                        let latest = Cli::checksum(&path, Some(default_branch.as_str()))?;
                        if latest != checksum {
                            self.logger.update(format!(
                                "[{name}] Fetching latest repository changes"
                            ));
                            if Cli::fetch(&path).is_err() {
                                self.logger.error(format!("[{name}] failed to fetch latest changes from git"));
                                continue;
                            };
                            self.logger.update(format!(
                                "[{name}] Setting branch to checksum `{latest}`"
                            ));
                            if Cli::reset(&path, ResetType::Hard, Some(latest)).is_err() {
                                self.logger.error(format!("[{name}] failed to set git branch"));
                                continue;
                            };
                        }
                    }
                    _ => {}
                },
            }

            self.logger.success(format!("{name} updated"));
            success += 1;
        }

        if self.config.write().is_err() {
            self.logger.error("failed to write updates to .luarc.json")
        }

        self.logger.success(format!("[Update] {success}/{} Finished!", addons.len()));

        Ok(())
    }

    pub fn clean(&mut self) -> Result<(), Error> {
        // Collect all that are in the config

        if self.base.join(ADDONS_DIR).exists() {
            for addon in (std::fs::read_dir(self.base.join(ADDONS_DIR))?).flatten() {
                if addon.path().is_dir()
                    && addon
                        .path()
                        .file_stem()
                        .map(|v| !self.config.get_addons().contains_key(&v.to_string_lossy()))
                        .unwrap_or_default()
                {
                    self.logger.update(format!(
                        "Removing unknown addon `{}`",
                        addon.path().file_stem().unwrap().to_string_lossy()
                    ));
                    std::fs::remove_dir_all(addon.path())
                        .map_err(Error::from)
                        .log_with(
                            &mut self.logger,
                            format!("failed to remove directory: {}", addon.path().display()),
                        );
                }
            }
        }

        self.logger.success("[Clean] Finished!");
        Ok(())
    }
}
