use zombienet_sdk::{NetworkConfig, NetworkConfigBuilder};

pub fn small_network() -> Result<NetworkConfig, anyhow::Error> {
    let config = NetworkConfigBuilder::new()
        .with_relaychain(|r| {
            r.with_chain("rococo-local")
                .with_default_command("polkadot")
                .with_default_image("docker.io/parity/polkadot:v1.7.0")
                .with_node(|node| node.with_name("alice"))
                .with_node(|node| node.with_name("bob"))
        })
        .with_parachain(|p| {
            p.with_id(2000).cumulus_based(true).with_collator(|n| {
                n.with_name("collator")
                    .with_command("polkadot-parachain")
                    .with_image("docker.io/parity/polkadot-parachain:1.7.0")
            })
        })
        .build()
        .unwrap();

    Ok(config)
}