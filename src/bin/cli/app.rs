use crate::{opt::Command, util::locate_project_root, Error, Options, Result};
use lib_pendulum_launch::{Config, Launcher};
use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
    process::{self, Output},
};
use structopt::StructOpt;

pub struct App(Options);

impl App {
    pub fn new(options: Options) -> Self {
        Self(options)
    }

    pub fn from_args() -> Self {
        Self::new(Options::from_args())
    }

    pub fn run(&mut self) -> Result<()> {
        match &self.0.cmd {
            Some(cmd) => match cmd {
                Command::ExportGenesis {
                    collator_bin,
                    collator_spec,
                    outdir,
                } => self.export_genesis(
                    collator_bin.to_owned(),
                    collator_spec.to_owned(),
                    outdir.to_owned(),
                )?,
                Command::GenerateSpecs { .. } => eprintln!("Unimplemented"),
            },
            None => self.launch()?,
        };

        Ok(())
    }

    fn launch(&mut self) -> Result<()> {
        let config = match &self.0.config {
            Some(config) => Some(config.to_owned()),
            None => search_default_config()?,
        };

        let launcher = match config {
            Some(path) => {
                let config = deserialize_config(path)?;
                Some(Launcher::from(config))
            }
            None => None,
        };

        match launcher {
            Some(mut launcher) => match launcher.run() {
                Ok(()) => Ok(()),
                Err(err) => Err(Error::Lib(err)),
            },
            None => Err(Error::InvalidPath),
        }
    }

    /// Export genesis data to an `outdir` if provided or to the project root
    fn export_genesis(&self, bin: PathBuf, chain: PathBuf, outdir: Option<PathBuf>) -> Result<()> {
        // Attempts to parse a PathBuf from a &str
        let path_to_str = |path: PathBuf| match path.to_str() {
            Some(path) => Ok(path.to_owned()),
            None => Err(Error::InvalidPath),
        };

        let bin = path_to_str(bin)?;
        let chain = path_to_str(chain)?;
        let outdir = {
            // Use project root if no `outdir` is provided
            let path = outdir.unwrap_or(locate_project_root()?);
            path_to_str(path)?
        };

        // Generates genesis data, given a name
        let generate = |name: &str| -> Result<()> {
            let cmd = format!("export-genesis-{name}");
            let output = process::Command::new(&bin)
                .args([&cmd, "--chain", &chain])
                .output()?;

            if !output.status.success() {
                return Err(Error::ProcessFailed(output.stderr));
            }

            let data = String::from_utf8(output.stdout)?;
            let out_file = format!("{outdir}/chain-{name}");
            fs::write(out_file, data)?;

            Ok(())
        };

        // Generate genesis-wasm and genesis-state
        ["wasm", "state"]
            .into_iter()
            .map(|name| generate(name))
            .collect()
    }
}

fn deserialize_config(path: PathBuf) -> Result<Config> {
    match Config::deserialize(path) {
        Ok(config) => Ok(config),
        Err(err) => Err(Error::Lib(err)),
    }
}

fn search_default_config() -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(".")? {
        if let Some(path) = try_get_config_entry(entry)? {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn try_get_config_entry(entry: io::Result<DirEntry>) -> Result<Option<PathBuf>> {
    let path = entry?.path();
    if path.is_file() {
        let path_name = path.as_os_str();
        if path_name == "launch.toml" || path_name == "launch.json" {
            return Ok(Some(path));
        }
    }

    Ok(None)
}
