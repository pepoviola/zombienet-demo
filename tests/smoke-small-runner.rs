// cargo test -p zombienet-sdk-demo --test smoke-small-runner -- basic_functionalities_should_works --exact --show-output

use futures::{stream::StreamExt, Future};
use serde_json::json;
use std::{panic, pin::Pin, time::Duration};
use zombienet_sdk::{LocalFileSystem, Network};
use zombienet_sdk::{NetworkConfig, NetworkConfigBuilder};

fn small_network() -> NetworkConfig {
    NetworkConfigBuilder::new()
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
        .unwrap()
}

pub fn run_test<T>(config: NetworkConfig, test: T)
where
    T: panic::UnwindSafe,
    T: FnOnce(Network<LocalFileSystem>) -> Pin<Box<dyn Future<Output = ()> + 'static + Send>>,
{
    use std::time::Instant;

    // let mut ns_name: Option<String> = None;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    let spawn_fn = zombienet_sdk_demo::environment::get_spawn_fn();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(async {
            let now = Instant::now();

            #[allow(unused_mut)]
            // let mut network = config.spawn_k8s().await.unwrap();
            let mut network = spawn_fn(config).await.unwrap();

            let elapsed = now.elapsed();
            log::info!("🚀🚀🚀🚀 network deployed in {:.2?}", elapsed);

            // get ns name to cleanup if test fails
            // ns_name = Some(network.ns_name());

            // run some tests on the newly started network
            test(network).await;
        })
    }));

    log::info!("result: {:?}", result);

    assert!(result.is_ok());
}

#[test]
fn basic_functionalities_should_works() {
    tracing_subscriber::fmt::init();
    let config = small_network();
    run_test(config, |network| {
        Box::pin(async move {
            // give some time to node bootstrap
            tokio::time::sleep(Duration::from_secs(10)).await;
            // Get a ref to the node
            let alice = network.get_node("alice").unwrap();

            let role = alice.reports("node_roles").await.unwrap();
            log::info!("Role is {role}");
            assert_eq!(role, 4.0);

            // subxt
            let client = alice.client::<subxt::PolkadotConfig>().await.unwrap();

            // wait 3 blocks
            let mut blocks = client.blocks().subscribe_finalized().await.unwrap().take(3);
            while let Some(block) = blocks.next().await {
                log::info!("Block #{}", block.unwrap().header().number);
            }

            // drop the client
            drop(client);

            // check best block through metrics
            let best_block = alice
                .reports("block_height{status=\"best\"}")
                .await
                .unwrap();

            assert!(best_block >= 2.0, "Current best {}", best_block);

            // pjs
            let para_is_registered = r#"
            const paraId = arguments[0];
            const parachains: number[] = (await api.query.paras.parachains()) || [];
            const isRegistered = parachains.findIndex((id) => id.toString() == paraId.toString()) >= 0;
            return isRegistered;
            "#;

            let is_registered = alice
                .pjs(para_is_registered, vec![json!(2000)], None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(is_registered, json!(true));

            // run pjs with code
            let query_paras = r#"
            const parachains: number[] = (await api.query.paras.parachains()) || [];
            return parachains.toJSON()
            "#;

            let paras = alice.pjs(query_paras, vec![], None).await.unwrap();

            log::info!("parachains registered: {:?}", paras);
        })
    });
}
