use crate::{opt::Command, Options};
use lib_pendulum_launch::{
    error::{Error, Result},
    sub_command, util, Config, Launcher,
};
use std::{
    fs::{self, DirEntry},
    io,
    path::PathBuf,
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
                    name,
                    outdir,
                } => self.export_genesis(
                    collator_bin.to_owned(),
                    collator_spec.to_owned(),
                    name.to_owned(),
                    outdir.to_owned(),
                )?,
                Command::GenerateSpecs {
                    collator_bin,
                    name,
                    para_id,
                    outdir,
                } => self.generate_specs(
                    collator_bin.to_owned(),
                    name.to_owned(),
                    para_id.to_owned(),
                    outdir.to_owned(),
                )?,
            },
            None => self.launch()?,
        };

        Ok(())
    }

    /// Launche parachain and idle until the program receives a `SIGINT`
    fn launch(&mut self) -> Result<()> {
        let (quiet, log) = (self.0.quiet, self.0.log.to_owned());

        if quiet && log.is_some() {
            return Err(Error::ProcessFailed(
                "Cannot use `--quiet` and `--log <DIR>` together".to_string(),
            ));
        }

        let mut config = deserialize_config(&self.0.config)?;
        Launcher::new(&mut config, log)?.run()
    }

    /// Export genesis data to an `outdir` if provided or to the project root
    fn export_genesis(
        &self,
        bin: PathBuf,
        chain: PathBuf,
        name: Option<String>,
        outdir: Option<PathBuf>,
    ) -> Result<()> {
        let bin = util::path_to_str(&bin)?;
        let chain = util::path_to_str(&chain)?;
        let name = name.unwrap_or_else(|| "local-chain".to_string());
        let outdir = util::path_to_str(&outdir.unwrap_or(util::locate_project_root()?))?;

        sub_command::export_genesis(bin, chain, name, outdir)
    }

    /// Generate specs from a collator
    fn generate_specs(
        &self,
        bin: PathBuf,
        name: Option<String>,
        para_id: Option<u32>,
        outdir: Option<PathBuf>,
    ) -> Result<()> {
        let bin = util::path_to_str(&bin)?;
        let name = name.unwrap_or_else(|| "local-chain".to_string());
        let para_id = para_id.unwrap_or(2000);
        let outdir = util::path_to_str(&outdir.unwrap_or(util::locate_project_root()?))?;

        sub_command::generate_specs(bin, name, para_id, outdir)
    }
}

/// Attempts to deserialize a config, searching for a default config if none is provided
fn deserialize_config(path: &Option<PathBuf>) -> Result<Config> {
    let path = {
        let path = match &path {
            Some(path) => Some(path.to_owned()),
            None => search_default_config()?,
        };

        match path {
            Some(path) => path,
            None => return Err(Error::NoConfig),
        }
    };

    Config::deserialize(path)
}

fn search_default_config() -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(util::locate_project_root()?)? {
        if let Some(path) = try_get_config_entry(entry)? {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn try_get_config_entry(entry: io::Result<DirEntry>) -> Result<Option<PathBuf>> {
    let path = entry?.path();
    match path.is_file() && util::path_to_str(&path)?.contains("launch.json") {
        true => Ok(Some(path)),
        false => Ok(None),
    }
}
