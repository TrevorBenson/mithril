use anyhow::Context;
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use config::{builder::DefaultState, ConfigBuilder};
use std::{collections::HashMap, sync::Arc};

use mithril_common::{messages::SnapshotMessage, StdResult};

use mithril_client::{
    dependencies::{ConfigParameters, DependenciesBuilder},
    SnapshotFieldItem,
};

/// Clap command to show a given snapshot
#[derive(Parser, Debug, Clone)]
pub struct SnapshotShowCommand {
    /// Enable JSON output.
    #[clap(long)]
    json: bool,

    /// Snapshot digest.
    ///
    /// If `latest` is specified as digest, the command will return the latest snapshot.
    digest: String,
}

impl SnapshotShowCommand {
    /// Snapshot Show command
    pub async fn execute(&self, config_builder: ConfigBuilder<DefaultState>) -> StdResult<()> {
        let config = config_builder.build()?;
        let params: Arc<ConfigParameters> = Arc::new(ConfigParameters::new(
            config.try_deserialize::<HashMap<String, String>>()?,
        ));
        let mut dependencies_builder = DependenciesBuilder::new(params);
        let snapshot_service = dependencies_builder
            .get_snapshot_service()
            .await
            .with_context(|| "Dependencies Builder can not get Snapshot Service")?;
        let snapshot_message = snapshot_service.show(&self.digest).await.with_context(|| {
            format!(
                "Snapshot Service can not show the snapshot for digest: '{}'",
                self.digest
            )
        })?;

        if self.json {
            println!("{}", serde_json::to_string(&snapshot_message)?);
        } else {
            print_stdout(convert_to_field_items(&snapshot_message).with_title()).unwrap();
        }

        Ok(())
    }
}

/// Convert Snapshot to SnapshotFieldItems routine
fn convert_to_field_items(snapshot_message: &SnapshotMessage) -> Vec<SnapshotFieldItem> {
    let mut field_items = vec![
        SnapshotFieldItem::new(
            "Epoch".to_string(),
            format!("{}", snapshot_message.beacon.epoch),
        ),
        SnapshotFieldItem::new(
            "Immutable File Number".to_string(),
            format!("{}", snapshot_message.beacon.immutable_file_number),
        ),
        SnapshotFieldItem::new(
            "Network".to_string(),
            snapshot_message.beacon.network.to_owned(),
        ),
        SnapshotFieldItem::new("Digest".to_string(), snapshot_message.digest.to_string()),
        SnapshotFieldItem::new("Size".to_string(), format!("{}", snapshot_message.size)),
        SnapshotFieldItem::new(
            "Cardano node version".to_string(),
            snapshot_message
                .cardano_node_version
                .as_ref()
                .unwrap_or(&"NA".to_string())
                .to_string(),
        ),
    ];
    for (idx, location) in snapshot_message.locations.iter().enumerate() {
        field_items.push(SnapshotFieldItem::new(
            format!("Location {}", idx + 1),
            location.to_string(),
        ));
    }
    field_items.push(SnapshotFieldItem::new(
        "Created".to_string(),
        snapshot_message.created_at.to_string(),
    ));
    field_items.push(SnapshotFieldItem::new(
        "Compression Algorithm".to_string(),
        format!(
            "{}",
            snapshot_message.compression_algorithm.unwrap_or_default()
        ),
    ));

    field_items
}
