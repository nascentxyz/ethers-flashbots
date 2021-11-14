use anyhow::Result;
use ethers::core::{rand::thread_rng, types::transaction::eip2718::TypedTransaction};
use ethers::prelude::*;
use ethers_flashbots::*;
use std::convert::TryFrom;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to the network
    let provider = Provider::<Http>::try_from("https://mainnet.eth.aragon.network")?;

    // This is your searcher identity
    let bundle_signer = LocalWallet::new(&mut thread_rng());

    // This signs transactions
    let wallet = LocalWallet::new(&mut thread_rng());

    // Add signer and Flashbots middleware
    let client = SignerMiddleware::new(
        FlashbotsMiddleware::new(
            provider,
            Url::parse("https://relay.flashbots.net")?,
            bundle_signer,
        ),
        wallet.clone(),
    );

    // Build a custom bundle that pays 0x0 (zero) one ether
    let mut eip1559_tx = TypedTransaction::Eip1559(
        Eip1559TransactionRequest::new()
            .from(wallet.address())
            .to(Address::zero())
            .value(1)
    );
    client.fill_transaction(&mut eip1559_tx, None).await?;

    // Sign the Transaction
    let signature = client.signer().sign_transaction(&eip1559_tx).await?;

    // Add the Transaction into a Bundle Request
    let bundle = BundleRequest::new()
        .push_transaction(eip1559_tx.rlp_signed(client.signer().chain_id(), &signature));

    // Simulate it
    let simulated_bundle = client.inner().simulate_bundle(&bundle).await?;
    println!("Simulated bundle: {:?}", simulated_bundle);

    // Send it
    let pending_bundle = client.inner().send_bundle(&bundle).await?;

    // You can also optionally wait to see if the bundle was included
    match pending_bundle.await {
        Ok(bundle_hash) => println!(
            "Bundle with hash {:?} was included in target block",
            bundle_hash
        ),
        Err(PendingBundleError::BundleNotIncluded) => {
            println!("Bundle was not included in target block.")
        }
        Err(e) => println!("An error occured: {}", e),
    }

    Ok(())
}
