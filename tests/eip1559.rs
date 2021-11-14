use anyhow::Result;
use ethers::core::{rand::thread_rng, types::transaction::eip2718::TypedTransaction};
use ethers::prelude::*;
use ethers_flashbots::*;
use std::convert::TryFrom;
use url::Url;

#[tokio::test]
async fn can_simulate_eip1559_bundled_transaction() {
    // Connect to the network
    let provider = match Provider::<Http>::try_from("https://mainnet.eth.aragon.network") {
      Ok(p) => p,
      Err(e) => {
        assert!(false, "Failed to create provider {:?}", e);
        return;
      }
    };

    // This is your searcher identity
    let bundle_signer = LocalWallet::new(&mut thread_rng());

    // This signs transactions
    let wallet = LocalWallet::new(&mut thread_rng());

    // Parse Flashbots Relay endpoint as url (this really shouldn't fail)
    let relay_url = match Url::parse("https://relay.flashbots.net") {
      Ok(url) => url,
      Err(e) => {
        assert!(false, "Failed to parse flashbots relay url {:?}", e);
        return;
      }
    };

    // Add signer and Flashbots middleware
    let client = SignerMiddleware::new(
        FlashbotsMiddleware::new(
            provider,
            relay_url,
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

    if let Err(e) = client.fill_transaction(&mut eip1559_tx, None).await {
      assert!(false, "Failed to fill transaction {}", e);
    }

    // Sign the Transaction
    let signature = match client.signer().sign_transaction(&eip1559_tx).await {
      Ok(sig) => sig,
      Err(e) => {
        assert!(false, "Failed to sign transaction {}", e);
        return;
      }
    };

    // Add the Transaction into a Bundle Request
    let bundle = BundleRequest::new()
        .push_transaction(eip1559_tx.rlp_signed(client.signer().chain_id(), &signature));

    // Simulate it
    match client.inner().simulate_bundle(&bundle).await {
      Ok(simulated_bundle) => {
        println!("Simulated bundle: {:?}", simulated_bundle);
        assert!(true, "Successfully simulated bundle with an EIP1559 Transaction!");
      }
      Err(e) => {
        assert!(false, "Failed to simulate bundle, error: {}", e);
      }
    }


}

    // // Send it
    // let pending_bundle = client.inner().send_bundle(&bundle).await?;

    // // You can also optionally wait to see if the bundle was included
    // match pending_bundle.await {
    //     Ok(bundle_hash) => println!(
    //         "Bundle with hash {:?} was included in target block",
    //         bundle_hash
    //     ),
    //     Err(PendingBundleError::BundleNotIncluded) => {
    //         println!("Bundle was not included in target block.")
    //     }
    //     Err(e) => println!("An error occured: {}", e),
    // }