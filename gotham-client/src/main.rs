// Gotham-city
//
// Copyright 2018 by Kzen Networks (kzencorp.com)
// Gotham city is free software: you can redistribute
// it and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//

use std::collections::HashMap;
use std::convert::TryFrom;

use bitcoin::hashes::hex::ToHex;
use clap::*;
use ethers::{
    core::{
        types::transaction::eip2718::TypedTransaction, types::TransactionRequest,
    },
    providers::{Http, Middleware, Provider},
    signers::Signer,
};
use ethers::prelude::*;
use eyre::Result;
use rand::rngs::mock::StepRng;
use sha2::{Digest, Sha256};

use client_lib::ClientShim;
use client_lib::wallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let yaml = load_yaml!("../cli.yml");
    // let matches = App::from_yaml(yaml).get_matches();
    // #[derive(Debug,Parser)]
    // #[clap(author,version,about)]
    // pub struct walletArgs{
    //     pub first:String,
    //     #[command(subcommand)]
    //     pub second:Action,
    // }

    #[derive(clap::Parser)]
    struct walletArgs {
        #[command(subcommand)]
        action: Action,
        // #[arg(short, long)]
        // name: Option<String>,
    }
    #[derive(clap::Subcommand)]
    enum Action {
        /// create a new wallet
        create,
        /// load an existing wallet
        load,
        /// drive new Address
        new,
        /// sign with the existing wallet
        // #[command(subcommand)]
        sign { address: String },
        /// sign an eth transaction
        eth { address: String },
    }
    let args = walletArgs::parse();
    let mut rng = StepRng::new(0, 1);

    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings"))
        .unwrap()
        // Add in settings from the environment (with prefix "APP")
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::new())
        .unwrap();
    let hm = settings.try_into::<HashMap<String, String>>().unwrap();
    let endpoint = hm.get("endpoint").unwrap();

    let client_shim = ClientShim::new(endpoint.to_string(), None);

    let network = "testnet".to_string();
    match &args.action {
        Action::create {} => {
            println!("'create: ");
            println!("Network: [{}], Creating wallet", network);
            let wallet = wallet::Wallet::new(&client_shim, &network);
            wallet.save();
            println!("Network: [{}], Wallet saved to disk", &network);
        }

        Action::new {} => {
            let mut wallet: wallet::Wallet = wallet::Wallet::load();
            println!("Load wallet: [{}]", wallet.id);
            let address = wallet.get_new_eth_address();
            println!("Network: [{}], Wallet: {} saved to disk", &network, "0x".to_owned() + &address.to_hex());
            wallet.save();
        }

        Action::load {} => {
            println!("'load: ");
            let mut wallet: wallet::Wallet = wallet::Wallet::load();
        }
        Action::sign { address } => {
            let mut wallet: wallet::Wallet = wallet::Wallet::load();
            println!("Load wallet: [{}]", wallet.id);
            println!("Sign: ");
            let mut msg_buf = "Test Signature";
            println!("message: [{}]", msg_buf);

            // create a Sha256 object
            let mut hasher = Sha256::new();

            // write input message
            hasher.update(msg_buf);

            // read hash digest and consume hasher
            let msg = hasher.finalize();
            wallet.sign(&msg, &address, &client_shim);
            println!("Network: [{}], MPC signature verified", &network);
        }
        Action::eth { address } => {
            const RPC_URL: &str = "https://eth.llamarpc.com";
            println!("'derive: ");
            let mut wallet: wallet::Wallet = wallet::Wallet::load();
            println!("Wallet: [{}], loaded", wallet.id);
            let provider = Provider::<Http>::try_from(RPC_URL)?;
            let block_number: U64 = provider.get_block_number().await?;
            println!("{block_number}");

            let tx: TypedTransaction = TransactionRequest {
                from: None,
                to: Some(
                    "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                        .parse::<Address>()
                        .unwrap()
                        .into(),
                ),
                value: Some(1_000_000_000.into()),
                gas: Some(2_000_000.into()),
                nonce: Some(0.into()),
                gas_price: Some(21_000_000_000u128.into()),
                data: None,
                chain_id: Some(U64::one()),
            }
                .into();
            // create a Sha256 object
            let mut hasher = Sha256::new();

            // write input message
            hasher.update(&tx.rlp());

            // read hash digest and consume hasher
            let msg = hasher.finalize();
            let transaction = serde_json::to_string(&tx).unwrap();
            println!("Transaction tx:{:?}", transaction);
            wallet.sign(&msg, &address, &client_shim);
        }
    }

    Ok(())
}
