use anyhow::Result;
use clap::Args;
use dojo_world::metadata::dojo_metadata_from_workspace;
use scarb::core::Config;

use super::options::account::AccountOptions;
use super::options::starknet::StarknetOptions;
use super::options::world::WorldOptions;
use crate::ops::migration;

#[derive(Args)]
pub struct MigrateArgs {
    #[arg(short, long)]
    #[arg(help = "Perform a dry run and outputs the plan to be executed.")]
    pub plan: bool,

    #[arg(long)]
    #[arg(help = "Name of the World.")]
    #[arg(long_help = "Name of the World. It's hash will be used as a salt when deploying the \
                       contract to avoid address conflicts.")]
    pub name: Option<String>,

    #[command(flatten)]
    pub world: WorldOptions,

    #[command(flatten)]
    pub starknet: StarknetOptions,

    #[command(flatten)]
    pub account: AccountOptions,
}

impl MigrateArgs {
    pub fn run(mut self, config: &Config) -> Result<()> {
        let ws = scarb::ops::read_workspace(config.manifest_path(), config)?;

        // If `name` was not specified use package name from `Scarb.toml` file if it exists
        if self.name.is_none() {
            if let Some(root_package) = ws.root_package() {
                self.name = Some(root_package.id.name.to_string());
            }
        }

        let target_dir = ws.target_dir().path_existent().unwrap();
        let target_dir = target_dir.join(ws.config().profile().as_str());

        if !target_dir.join("manifest.json").exists() {
            let packages = ws.members().map(|p| p.id).collect();
            scarb::ops::compile(packages, &ws)?;
        }

        let env_metadata = dojo_metadata_from_workspace(&ws).and_then(|inner| inner.env().cloned());
        // TODO: Check the updated scarb way to read profile specific values

        ws.config().tokio_handle().block_on(migration::execute(
            self,
            env_metadata,
            target_dir,
            ws.config(),
        ))?;

        Ok(())
    }
}
