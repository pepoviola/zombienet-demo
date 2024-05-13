use std::time::Duration;

use subxt::{OnlineClient, PolkadotConfig};
use zombienet_sdk::NetworkNode;

pub async fn sleep(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

pub async fn wait_subxt_client(
    node: &NetworkNode,
) -> Result<subxt::OnlineClient<PolkadotConfig>, anyhow::Error> {
    log::debug!("trying to connect to: {}", node.ws_uri());
    loop {
        let res: Result<OnlineClient<PolkadotConfig>, anyhow::Error> =
            match node.client::<subxt::PolkadotConfig>().await {
                Ok(cli) => {
                    break Ok(cli);
                }
                Err(e) => {
                    let cause = e.to_string();
                    log::trace!("{:?}", e);
                    if let subxt::Error::Rpc(subxt::error::RpcError::ClientError(inner)) = e {
                        log::trace!("inner: {}", inner.to_string());
                        let inner_str = inner.to_string();
                        if inner_str.contains("i/o error") || inner_str.contains("Connection refused") {
                            // The node is not ready to accept connections yet
                            sleep(1).await;
                            continue;
                        }
                    }
                    Err(anyhow::anyhow!("Cannot connect to node : {:?}", cause))?
                }
            };

        return res;
    }
}

pub async fn wait_for_metric(node: &NetworkNode, metric: &str, value: u64) -> Result<(), anyhow::Error> {
	log::info!("Waiting for {metric} to reach {value}:");
	loop {
		let current = node.reports(metric).await.unwrap_or(0.0) as u64;
		println!("{metric} = {current}");
		if current >= value {
			return Ok(());
		}
		// sleep at least one second
        sleep(1).await;
	}
}
