// cargo nextest archive --archive-file smoke.tar.zst
// cargo nextest run --archive-file ./smoke.tar.zst --no-capture -- zombienet_sdk_demo::smoke::test_smoke
use futures::stream::StreamExt;

#[tokio::test(flavor = "multi_thread")]
async fn test_smoke() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    let config = zombienet_sdk_demo::config::small_network().unwrap();
    let spawn_fn = zombienet_sdk_demo::environment::get_spawn_fn();
    let network = spawn_fn(config).await.unwrap();

    let alice = network.get_node("alice").unwrap();
    let client = zombienet_sdk_demo::waiting_helpers::wait_subxt_client(alice).await.unwrap();

    // wait 3 blocks
    let mut blocks = client.blocks().subscribe_finalized().await.unwrap().take(3);
    while let Some(block) = blocks.next().await {
        log::info!("Block #{}", block.unwrap().header().number);
    }

    Ok(())
}