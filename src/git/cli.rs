use std::path::Path;

use crate::Error;

pub enum ResetType {
    Soft,
    Hard
}

impl AsRef<str> for ResetType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Soft => "soft",
            Self::Hard => "hard"
        }
    }
}

pub struct Cli;
impl Cli {
    pub fn checksum<P: AsRef<Path>>(dir: P, branch: Option<&str>) -> Result<String, Error> {
        let result = if let Some(branch) = branch.as_ref() {
            //git log -n 1 origin/main --pretty=format:'%H'
            std::process::Command::new("git") 
                .args(["log", "-n", "1", format!("origin/{branch}").as_str(), "--pretty=format:'%H'"])
                .current_dir(dir)
                .output()?
        } else {
            std::process::Command::new("git") 
                .args(["rev-parse", "--verify", "HEAD"])
                .current_dir(dir)
                .output()?
        };

        if !result.status.success() { 
            return Err(Error::custom(format!("Failed to get latest checksum:\n{}", String::from_utf8_lossy(&result.stderr))))
        }
        Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
    }

    pub fn branch_name<P: AsRef<Path>>(dir: P) -> Result<String, Error> {
        let result = std::process::Command::new("git") 
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()?;

        Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
    }

    pub fn default_branch_name<P: AsRef<Path>>(dir: P) -> Result<String, Error> {
        let result = std::process::Command::new("git") 
            .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
            .current_dir(dir)
            .output()?;

        let result = String::from_utf8_lossy(&result.stdout).trim().to_string();
        Ok(result.rsplit_once('/').unwrap().1.to_string())
    }

    pub fn fetch<P: AsRef<Path>>(dir: P) -> Result<(), Error> {
        std::process::Command::new("git") 
            .args(["fetch", "-p"])
            .current_dir(dir)
            .output()?;

        Ok(())
    }

    pub fn switch<P: AsRef<Path>>(dir: P, branch: impl AsRef<str>) -> Result<(), Error> {
        std::process::Command::new("git") 
            .args(["switch", branch.as_ref()])
            .current_dir(dir)
            .output()?;

        Ok(())
    }

    pub fn pull<P: AsRef<Path>>(dir: P, force: bool) -> Result<(), Error> {
        let mut args = vec!["pull"];
        if force {
            args.push("--force");
        }

        std::process::Command::new("git") 
            .args(args)
            .current_dir(dir)
            .output()?;

        Ok(())
    }

    pub fn reset<P: AsRef<Path>, S: AsRef<str>>(dir: P, ty: ResetType, target: Option<S>) -> Result<(), Error> {
        let mut args = vec!["pull", ty.as_ref()];
        if let Some(target) = target.as_ref() {
            args.push(target.as_ref());
        }

        std::process::Command::new("git") 
            .args(args)
            .current_dir(dir)
            .output()?;

        Ok(())
    }

    pub fn clone(dir: impl AsRef<Path>, url: impl AsRef<str>, name: impl AsRef<str>) -> Result<(), Error> {
        let result = std::process::Command::new("git") 
            .args(["clone", url.as_ref(), name.as_ref()])
            .current_dir(dir)
            .output()?;

        if result.status.success() {
            Ok(()) 
        } else {
            Err(Error::custom(String::from_utf8_lossy(&result.stderr).trim()))
        }
    }
}
