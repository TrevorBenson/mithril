use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use config::{builder::DefaultState, ConfigBuilder};
use mithril_common::StdResult;

use mithril_client::{
    dependencies::{ConfigParameters, DependenciesBuilder},
    MithrilStakeDistributionListItem,
};

/// Mithril stake distribution LIST command
#[derive(Parser, Debug, Clone)]
pub struct MithrilStakeDistributionListCommand {
    /// Enable JSON output.
    #[clap(long)]
    json: bool,
}

impl MithrilStakeDistributionListCommand {
    /// Main command execution
    pub async fn execute(&self, config_builder: ConfigBuilder<DefaultState>) -> StdResult<()> {
        let config = config_builder.build()?;
        let params: Arc<ConfigParameters> = Arc::new(ConfigParameters::new(
            config.try_deserialize::<HashMap<String, String>>()?,
        ));
        let mut dependencies_builder = DependenciesBuilder::new(params);
        let service = dependencies_builder
            .get_mithril_stake_distribution_service()
            .await
            .with_context(|| {
                "Dependencies Builder can not get Mithril Stake Distribution Service"
            })?;
        let lines = service.list().await.with_context(|| {
            "Mithril Stake Distribution Service can not get the list of artifacts"
        })?;

        if self.json {
            println!("{}", serde_json::to_string(&lines)?);
        } else {
            let lines = lines
                .into_iter()
                .map(MithrilStakeDistributionListItem::from)
                .collect::<Vec<_>>();
            print_stdout(lines.with_title())?;
        }

        Ok(())
    }
}
