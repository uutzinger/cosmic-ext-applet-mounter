// SPDX-License-Identifier: MIT

use cosmic_ext_applet_mounter::diagnostics::DependencyInventory;
use cosmic_ext_applet_mounter::process::SystemCommandRunner;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let inventory =
        DependencyInventory::inspect(&SystemCommandRunner, CancellationToken::new()).await;

    for report in inventory.reports {
        println!(
            "{:?}: {:?}; version={}; path={}; {}",
            report.kind,
            report.state,
            report
                .version
                .map_or_else(|| "unknown".into(), |version| version.to_string()),
            report.path.as_deref().unwrap_or("not found"),
            report.summary
        );
    }
}
