// Gotham-city
//
// Copyright 2018 by Kzen Networks (kzencorp.com)
// Gotham city is free software: you can redistribute
// it and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//

use std::collections::HashMap;
use std::env;
use std::fs;
use std::str;
use std::str::FromStr;

use cfg_if::cfg_if;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use dirs::document_dir;

use bitcoin::hashes::hex::ToHex;
use hex;
use kms::ecdsa::two_party::*;
use kms::ecdsa::two_party::MasterKey2;
use log::debug;
// use secp256k1::{ecdsa::Signature, Message, SECP256K1,PublicKey,ecdsa::RecoveryId,ecdsa::};
use secp256k1::{
    ecdsa::Signature, Message, Secp256k1,
};
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::Digest;
use sha3::Keccak256;
pub use two_party_ecdsa::curv::{arithmetic::traits::Converter, BigInt};
use two_party_ecdsa::curv::elliptic::curves::traits::ECPoint;
use two_party_ecdsa::party_one::SignatureRecid;
use uuid::Uuid;

use crate::Client;

use super::ClientShim;
use super::ecdsa;
use super::ecdsa::types::PrivateShare;

const WALLET_FILENAME: &str = "mywallet";

use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static GOTHAM_ANDROID_PATH: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::from("mywallet")));


#[derive(Serialize, Deserialize)]
pub struct SignSecondMsgRequest {
    pub message: BigInt,
    pub party_two_sign_message: party2::SignMessage,
    pub pos_child_key: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AddressDerivation {
    pub pos: u32,
    pub mk: MasterKey2,
}

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub network: String,
    pub private_share: PrivateShare,
    pub last_derived_pos: u32,
    pub addresses_derivation_map: HashMap<String, AddressDerivation>,
}

impl Wallet {
    pub fn new<C: Client>(client_shim: &ClientShim<C>, net: &str) -> Wallet {
        let id = Uuid::new_v4().to_string();
        let private_share = ecdsa::get_master_key(client_shim);
        let last_derived_pos = 0;
        let addresses_derivation_map = HashMap::new();
        let network = net;
        Wallet {
            id,
            network: network.to_string(),
            private_share,
            last_derived_pos,
            addresses_derivation_map,
        }
    }

    pub fn get_new_eth_address(&mut self) -> ethers::types::Address {
        let (pos, mk) = Self::derive_new_key(&self.private_share, self.last_derived_pos);
        let pk = mk.public.q.get_element();
        let pk_bytes = pk.serialize_uncompressed();
        let hash = hex::encode(Keccak256::digest(&pk_bytes[1..]));
        let address_str = &hash[hash.len() - 40..];
        let address: ethers::types::Address = address_str.parse().unwrap();
        self.addresses_derivation_map
            .insert(address.to_hex(), AddressDerivation { mk, pos });
        self.last_derived_pos = pos;

        return address;
    }
    

    pub fn get_app_sandbox_path() -> Option<PathBuf> {
        cfg_if! {
            if #[cfg(target_os = "ios")] {
                // iOS 沙盒路径获取逻辑
                if let Ok(home_dir) = env::var("HOME") {
                    let app_sandbox_dir = PathBuf::from(format!("{}/Documents/mpc_wallet_plugin", home_dir));
                    Some(app_sandbox_dir)
                } else {
                    None
                }
            } else if #[cfg(target_os = "android")] {
                // Android 沙盒路径获取逻辑
                let android_path = GOTHAM_ANDROID_PATH.read().expect("Could not acquire read lock");
                let app_sandbox_dir = PathBuf::from(format!("{}/mpc_wallet_plugin", android_path));
                return Some(app_sandbox_dir);
            } else if #[cfg(target_os = "linux")] {
                // Linux 沙盒路径获取逻辑
                if let Ok(home_dir) = env::var("HOME") {
                    let app_sandbox_dir = PathBuf::from(format!("{}/.sandbox/mpc_wallet_plugin", home_dir));
                    Some(app_sandbox_dir)
                } else {
                    None
                }
            } else if #[cfg(target_os = "macos")] {
                // macOS 沙盒路径获取逻辑
                if let Ok(home_dir) = env::var("HOME") {
                    let app_sandbox_dir = PathBuf::from(format!("{}/Library/Sandbox/mpc_wallet_plugin", home_dir));
                    Some(app_sandbox_dir)
                } else {
                    None
                }
            } else if #[cfg(target_os = "windows")] {
                // Windows 沙盒路径获取逻辑
                if let Ok(app_data_dir) = env::var("APPDATA") {
                    let app_sandbox_dir = PathBuf::from(format!("{}/sandbox/mpc_wallet_plugin", app_data_dir));
                    Some(app_sandbox_dir)
                } else {
                    None
                }
            } else {
                None // 其他操作系统暂不支持
         
            }
        }
    }

    pub fn save_to(&self, filename: &str) {
        let wallet_json = serde_json::to_string(self).unwrap();
        
        if let Some(mut documents_dir) = Self::get_app_sandbox_path() {
            documents_dir.push(filename);
            
            let save_path = Path::new(&documents_dir);
            let mut file = match File::create(save_path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("save_to Unable to create file {:?}: {}", save_path, err);
                    return;
                }
            };
            if let Err(err) = file.write_all(wallet_json.as_bytes()) {
                eprintln!("Unable to save wallet to {:?}: {}", save_path, err);
                return;
            }
            println!("Saved wallet to disk: {:?}", save_path);
        } else {
            println!("Unable to get documents directory");
        }
    }

    pub fn save(&self) {
        self.save_to(WALLET_FILENAME)
    }

    pub fn save_file(&self, file_name: String) {
        self.save_to(file_name.as_str())
    }

    pub fn load_from(filepath: &str) -> Wallet {
        info!("load_from filepath = {}",filepath);
        if let Some(mut documents_dir) = Self::get_app_sandbox_path() {
            let load_path = documents_dir.join(filepath);

            if let Some(path_str) = load_path.to_str() {
                println!("Path value: {}", path_str);
            } else {
                println!("Failed to convert PathBuf to str");
            }

            if let Ok(data) = fs::read_to_string(load_path.clone()) {
                let wallet: Wallet = serde_json::from_str(&data).unwrap();
                debug!("(wallet id: {}) Loaded wallet to memory", wallet.id);
                return wallet;
            } else {
                panic!("Unable to load wallet from {}", load_path.to_string_lossy());
            }
        } else {
            panic!("Unable to get documents directory");
        }
    }

    pub fn load() -> Wallet {
        Wallet::load_from(WALLET_FILENAME)
    }

    pub fn loadwith(file_name: String) -> Wallet {
        Wallet::load_from(file_name.as_str())
    }

    pub fn sign<C: Client>(
        &mut self,
        msg: &[u8],
        address: &str,
        client_shim: &ClientShim<C>,
    ) -> Option<SignatureRecid> {
        if !self.addresses_derivation_map.contains_key(address) {
            return None;
        }
        let key = self.addresses_derivation_map.get(address).unwrap();
        let signature = ecdsa::sign(
            client_shim,
            BigInt::from(&msg[..]),
            &key.mk,
            BigInt::from(0),
            BigInt::from(key.pos),
            &self.private_share.id,
        )
            .expect("ECDSA signature failed");
        let r = BigInt::to_vec(&signature.r);
        let s = BigInt::to_vec(&signature.s);
        let message = Message::from_slice(msg).unwrap();
        println!(
            "hash{:?},\nsignature: [r={},s={}]",
            msg, &signature.r, &signature.s
        );
        return Some(signature);

        //prepare signature to be verified from secp256k1 lib

        // let mut sig = [0u8; 64];
        // sig[32 - r.len()..32].copy_from_slice(&r);
        // sig[32 + 32 - s.len()..].copy_from_slice(&s);
        //
        // let Sig = Signature::from_compact(&sig).unwrap();
        // let pk = child_master_key.public.q.get_element();
        //
        // let secp = Secp256k1::new();
        // //v = chain_id * 2 + 35 + recovery_id
        // let id = secp256k1::ecdsa::RecoveryId::from_i32(signature.recid as i32).unwrap();
        // let sig = secp256k1::ecdsa::RecoverableSignature::from_compact(&sig, id).unwrap();
        //
        // assert_eq!(secp.recover_ecdsa(&message, &sig), Ok(pk));
        //
        // println!("Trying to recover pk from r,s,recid");
        // println!("Recovered pk:{:?}", secp.recover_ecdsa(&message, &sig));
        // println!("pk:{:?}", pk);
    }

    fn derive_new_key(private_share: &PrivateShare, pos: u32) -> (u32, MasterKey2) {
        let last_pos: u32 = pos + 1;

        let last_child_master_key = private_share
            .master_key
            .get_child(vec![BigInt::from(0), BigInt::from(last_pos)]);

        (last_pos, last_child_master_key)
    }
}
