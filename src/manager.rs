use std::{borrow::Cow, path::{Path, PathBuf}};

use spinoff::{spinners, Spinner, Color};

use crate::{config::{LuaRc, Workspace}, error::UpdateSpinner, git::{ResetType, Cli}, Addon, Error, ADDONS_DIR};

pub const DOTS: spinners::Dots = spinners::Dots;

#[derive(Debug)]
pub struct Manager {
    pub base: PathBuf,
    pub config: LuaRc,
}

impl Manager {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let path = dir.as_ref();
        Ok(Self {
            config: LuaRc::detect(path)?,
            base: path.to_path_buf(),
        })
    }

    pub fn clone_addon(&mut self, name: Cow<'static, str>) -> Result<(), Error> {
        // PERF: Return error or log when addon is not in lock file
        if let Some(addon) = self.config.get_addons().get(&name) {
            let temp_name = addon.checksum.clone().unwrap_or(uuid::Uuid::now_v7().to_string());
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

    pub fn add(&mut self, addons: Vec<Addon>) -> Result<(), Error> {
        let total = addons.len().to_string();
        let mut success = 0;
        let mut spinner = Spinner::new(DOTS, format!("{:0>width$}/{total} Cloning Addons", success, width=total.len()), Color::Yellow); 

        let addon_path = self.base.join(ADDONS_DIR);
        for addon in addons.iter() {
            let name = addon.name();
            let path = addon_path.join(name.as_ref());
            spinner.update_text(format!("{:0>width$}/{total} Cloning {name}", success, width=total.len()));

            if !path.exists() || !self.config.get_addons().contains_key(name.as_ref()) {
                self.config.update_addon(addon);
                if let Err(err) = self.clone_addon(name.clone()) {
                    err.update_spinner(
                        &mut spinner,
                        format!("failed to clone addon: {name}")
                    );
                    continue;
                }
            } else {
                let branch_diff = addon.branch.as_ref().map(|v| Cli::branch_name(&path).map(|n| &n != v).unwrap_or_default()).unwrap_or_default();
                let checksum_diff = addon.checksum.as_ref().map(|v| Cli::checksum(&path, None).map(|n| &n != v).unwrap_or_default()).unwrap_or_default();
                self.config.update_addon(addon);

                if branch_diff || checksum_diff {
                    if let Err(err) = self.clone_addon(name.clone()) {
                        err.update_spinner(
                            &mut spinner,
                            format!("failed to clone addon: {name}")
                        );
                        continue;
                    }
                }
            };
            success += 1;
        }

        spinner.update_text("Updating .luarc.json");

        let path = ADDONS_DIR.to_string();
        match self.config.workspace.as_mut() {
            Some(workspace) => {
                if !workspace.user_third_party.contains(&path) {
                    workspace.user_third_party.push(path);
                }
            },
            None => {
                self.config.workspace = Some(Workspace {
                    user_third_party: Vec::from([path]),
                    ..Default::default()
                });
            }
        }

        if let Err(err) = self.config.write() {
            err.update_spinner(
                &mut spinner,
                "failed to write updates to .luarc.json",
            )
        }

        spinner.success(format!("[Add] {success}/{total} Finished!").as_str());
        
        Ok(())
    }

    pub fn remove(&mut self, mut addons: Vec<Addon>, all: bool) -> Result<(), Error> {
        if all {
            addons = self.config.get_addons().values().cloned().collect();    
        }

        let total = addons.len().to_string();
        let mut spinner = Spinner::new(DOTS, format!("{:0>width$}/{total} Removing ...", 0, width=total.len()), Color::Yellow); 

        let addon_path = self.base.join(ADDONS_DIR);
        for (i, addon) in addons.iter().enumerate() {
            let name = addon.name();
            let path = addon_path.join(name.as_ref());
            spinner.update_text(format!("{:0>width$}/{total} Removing {name}", i+1, width=total.len()));

            if self.config.get_addons().contains_key(name.as_ref()) {
                self.config.get_addons_mut().remove(name.as_ref());
            }

            if path.exists() {
                std::fs::remove_dir_all(path)?;
            }
        }

        if let Err(err) = self.config.write() {
            err.update_spinner(
                &mut spinner,
                "failed to write updates to .luarc.json",
            )
        }

        spinner.success(format!("[Remove] {total}/{total} Finished!").as_str());
        
        Ok(())
    }

    pub fn update(&mut self, mut addons: Vec<Addon>, all: bool) -> Result<(), Error> {
        // Collect all that are in the config
        if all {
            addons = self.config.get_addons().values().cloned().collect();    
        }

        let mut spinner = Spinner::new(DOTS, "", Color::Yellow); 

        let addon_path = self.base.join(ADDONS_DIR);
        for addon in addons.iter() {
            let name = addon.name();

            if !self.config.get_addons().contains_key(name.as_ref()) {
                continue
            }
            self.config.update_addon(addon);
            let addon = self.config.get_addons().get(&name).unwrap();

            let path = addon_path.join(name.as_ref());

            spinner.update_text(format!("[{name}] Getting branch name"));
            let branch = Cli::branch_name(&path)?;

            spinner.update_text(format!("[{name}] Getting default branch name"));
            let default_branch = Cli::default_branch_name(&path)?;

            spinner.update_text(format!("[{name}] Getting current checksum"));
            let checksum = Cli::checksum(&path, None)?;

            match addon.branch.as_ref() {
                Some(b) if b != &branch => {
                    spinner.update_text(format!("[{name}] Fetching latest repository changes"));
                    Cli::fetch(&path).ok_with_spinner(
                        &mut spinner,
                        format!("[{name}] failed to fetch latest changes from git"),
                    );
                    
                    spinner.update_text(format!("[{name}] Switching to branch `{b}`"));
                    Cli::switch(&path, b).ok_with_spinner(
                        &mut spinner,
                        format!("[{name}] failed to switch git branches"),
                    );
                    
                    spinner.update_text(format!("[{name}] Pulling latest changes"));
                    Cli::pull(&path, false).ok_with_spinner(
                        &mut spinner,
                        format!("[{name}] failed to pull latest changes")
                    );
                    
                    if let Some(checksum) = addon.checksum.as_deref() {
                        spinner.update_text(format!("[{name}] Setting branch to checksum `{checksum}`"));
                        Cli::reset(&path, ResetType::Hard, Some(checksum)).ok_with_spinner(
                            &mut spinner,
                            format!("[{name}] failed to set git branch")
                        );
                    }
                },
                None if branch != default_branch => {
                    spinner.update_text(format!("[{name}] Fetching latest repository changes"));
                    Cli::fetch(&path).ok_with_spinner(
                        &mut spinner,
                        format!("[{name}] failed to fetch latest changes from git"),
                    );
                    
                    spinner.update_text(format!("[{name}] Switching to branch `{default_branch}`"));
                    Cli::switch(&path, &default_branch).ok_with_spinner(
                        &mut spinner,
                        format!("[{name}] failed to switch git branches"),
                    );
                    
                    spinner.update_text(format!("[{name}] Pulling latest changes"));
                    Cli::pull(&path, false).ok_with_spinner(
                        &mut spinner,
                        format!("[{name}] failed to pull latest changes")
                    );
                    
                    if let Some(checksum) = addon.checksum.as_deref() {
                        spinner.update_text(format!("[{name}] Setting branch to checksum `{checksum}`"));
                        Cli::reset(&path, ResetType::Hard, Some(checksum)).ok_with_spinner(
                            &mut spinner,
                            format!("[{name}] failed to set git branch")
                        );
                    }
                },
                _ => match addon.checksum.as_ref() {
                    Some(c) if c != &checksum => {
                        spinner.update_text(format!("[{name}] Fetching latest repository changes"));
                        Cli::fetch(&path).ok_with_spinner(
                            &mut spinner,
                            format!("[{name}] failed to fetch latest changes from git"),
                        );
                        spinner.update_text(format!("[{name}] Setting branch to checksum `{c}`"));
                        Cli::reset(&path, ResetType::Hard, Some(c)).ok_with_spinner(
                            &mut spinner,
                            format!("[{name}] failed to set git branch")
                        );
                    },
                    None => {
                        let latest = Cli::checksum(&path, Some(default_branch.as_str()))?;
                        if latest != checksum {
                            spinner.update_text(format!("[{name}] Fetching latest repository changes"));
                            Cli::fetch(&path).ok_with_spinner(
                                &mut spinner,
                                format!("[{name}] failed to fetch latest changes from git"),
                            );
                            spinner.update_text(format!("[{name}] Setting branch to checksum `{latest}`"));
                            Cli::reset(&path, ResetType::Hard, Some(latest)).ok_with_spinner(
                                &mut spinner,
                                format!("[{name}] failed to set git branch")
                            );
                        }
                    },
                    _ => {}
                }
            }
        }

        if let Err(err) = self.config.write() {
            err.update_spinner(
                &mut spinner,
                "failed to write updates to .luarc.json",
            )
        }

        spinner.success("[Update] Finished!");
        
        Ok(())
    }

    pub fn clean(&mut self) -> Result<(), Error> {
        // Collect all that are in the config
        let mut spinner = Spinner::new(DOTS, "", Color::Yellow); 

        if self.base.join(ADDONS_DIR).exists() {
            for addon in (std::fs::read_dir(self.base.join(ADDONS_DIR))?).flatten() {
                if addon.path().is_dir() && addon.path().file_stem().map(|v| !self.config.get_addons().contains_key(&v.to_string_lossy())).unwrap_or_default() {
                    spinner.update_text(format!("Removing unknown addon `{}`", addon.path().file_stem().unwrap().to_string_lossy()));
                    std::fs::remove_dir_all(addon.path())
                        .map_err(Error::from)
                        .ok_with_spinner(
                            &mut spinner,
                            format!("failed to remove directory: {}", addon.path().display())
                        );
                }
            }
        }
        
        spinner.success("[Clean] Finished!");

        Ok(())
    }
}
