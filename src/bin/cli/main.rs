mod app;
mod opt;
mod util;

use crate::opt::Command;
use lib_pendulum_launch::{sub_command, Error, Launcher, Result};
use opt::Options;
use std::{io, path::PathBuf};
use structopt::StructOpt;

fn main() -> Result<()> {
    App::from_args().run()
}

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
                Command::GenerateDocker {
                    outdir,
                    enable_volume,
                } => self.generate_docker(outdir.to_owned(), *enable_volume)?,
            },
            None => self.launch()?,
        };

        Ok(())
    }

    /// Launch parachain and await SIGINT
    fn launch(&mut self) -> Result<()> {
        let (quiet, log) = (self.0.quiet, self.0.log.to_owned());

        if quiet && log.is_some() {
            return Err(Error::ProcessFailed(
                "Cannot use `--quiet` and `--log <DIR>` together".to_string(),
            ));
        }

        let mut config = util::deserialize_config(&self.0.config)?;
        config.ensure_unique_ports()?;

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
        let bin = lib_pendulum_launch::util::path_to_string(&bin)?;
        let chain = lib_pendulum_launch::util::path_to_string(&chain)?;
        let name = name.unwrap_or_else(|| "local-chain".to_string());
        let outdir = lib_pendulum_launch::util::path_to_string(
            &outdir.unwrap_or(lib_pendulum_launch::util::locate_project_root()?),
        )?;

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
        let bin = lib_pendulum_launch::util::path_to_string(&bin)?;
        let name = name.unwrap_or_else(|| "local-chain".to_string());
        let para_id = para_id.unwrap_or(2000);
        let outdir = lib_pendulum_launch::util::path_to_string(
            &outdir.unwrap_or(lib_pendulum_launch::util::locate_project_root()?),
        )?;

        sub_command::generate_specs(bin, name, para_id, outdir)
    }

    fn generate_docker(&self, out_dir: Option<PathBuf>, enable_volume: bool) -> Result<()> {
        let config = util::deserialize_config(&self.0.config)?;
        config.ensure_unique_ports()?;
        let out_dir = lib_pendulum_launch::util::path_to_string(
            &out_dir.unwrap_or(lib_pendulum_launch::util::locate_project_root()?),
        )?;

        let command = sub_command::GenerateDocker::new(config, out_dir, enable_volume);
        command.execute()
    }
}

// /// Attempts to deserialize a config, searching for a default config if none is provided
// fn deserialize_config(path: &Option<PathBuf>) -> Result<Config> {
//     let path = {
//         let path = match &path {
//             Some(path) => Some(path.to_owned()),
//             None => search_default_config()?,
//         };

//         match path {
//             Some(path) => path,
//             None => return Err(Error::NoConfig),
//         }
//     };

//     Config::deserialize(path)
// }

// fn search_default_config() -> Result<Option<PathBuf>> {
//     for entry in fs::read_dir(util::locate_project_root()?)? {
//         if let Some(path) = try_get_config_entry(entry)? {
//             return Ok(Some(path));
//         }
//     }

//     Ok(None)
// }

// fn try_get_config_entry(entry: io::Result<DirEntry>) -> Result<Option<PathBuf>> {
//     let path = entry?.path();
//     match path.is_file() && util::path_to_string(&path)?.contains("launch.json") {
//         true => Ok(Some(path)),
//         false => Ok(None),
//     }
// }
