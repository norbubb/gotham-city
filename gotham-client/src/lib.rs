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
// use eyre::Result;
use rand::rngs::mock::StepRng;
use sha2::{Digest, Sha256};

// use ClientShim;
// use wallet;

use floating_duration::TimeFormat;
use log::info;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Instant;
pub mod ecdsa;
pub mod escrow;

// ffi bindings
use std::ffi::{CStr, CString, c_longlong};
use std::os::raw::c_char;


// extern crate config;
extern crate kms;
extern crate reqwest;

#[macro_use]
// extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate log;

#[macro_use]
extern crate failure;
//
// extern crate bitcoin;
// extern crate electrumx_client;
// extern crate hex;
// extern crate itertools;
// extern crate uuid;

mod utilities;
pub mod wallet;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug)]
pub struct ClientShim<C: Client> {
    pub client: C,
    pub auth_token: Option<String>,
    pub endpoint: String,
}

impl ClientShim<reqwest::Client> {
    pub fn new(endpoint: String, auth_token: Option<String>) -> ClientShim<reqwest::Client> {
        let client = reqwest::Client::new();
        ClientShim {
            client,
            auth_token,
            endpoint,
        }
    }
    
}

impl<C: Client> ClientShim<C> {
    pub fn new_with_client(endpoint: String, auth_token: Option<String>, client: C) -> Self {
        Self {
            client,
            auth_token,
            endpoint,
        }
    }
    pub fn post<V>(&self, path: &str) -> Option<V>
    where
        V: serde::de::DeserializeOwned,
    {
        let start = Instant::now();
        println!("ClientShim -- post path = {}",path);
        let res = self
            .client
            .post(&self.endpoint, path, self.auth_token.clone(), "{}");
        info!("(req {}, took: {:?})", path, TimeFormat(start.elapsed()));
        res
    }

    pub fn postb<T, V>(&self, path: &str, body: T) -> Option<V>
    where
        T: serde::ser::Serialize,
        V: serde::de::DeserializeOwned,
    {
        let start = Instant::now();

        let res = self
            .client
            .post(&self.endpoint, path, self.auth_token.clone(), body);
        info!("(req {}, took: {:?})", path, TimeFormat(start.elapsed()));
        res
    }


    
}


impl<C> ClientShim<C>
where
    C: Client,
{
    pub fn test(&self, flag: i32) {
        // 在这里添加 test 方法的具体实现
        println!("Testing ClientShim - flag: {}", flag);
    }
}

pub trait Client: Sized {
    fn post<V: DeserializeOwned, T: Serialize>(
        &self,
        endpoint: &str,
        uri: &str,
        bearer_token: Option<String>,
        body: T,
    ) -> Option<V>;
}

impl Client for reqwest::Client {
    fn post<V: DeserializeOwned, T: Serialize>(
        &self,
        endpoint: &str,
        uri: &str,
        bearer_token: Option<String>,
        body: T,
    ) -> Option<V> {

        println!("Client - request -post complete path = {}", format!("{}/{}", endpoint, uri));
        let mut b = self.post(&format!("{}/{}", endpoint, uri));
        if let Some(token) = bearer_token {
            b = b.bearer_auth(token);
        }
        let value: String = b.json(&body).send().ok()?.text().ok()?;
        serde_json::from_str(value.as_str()).ok()
    }
}

pub use two_party_ecdsa::curv::{arithmetic::traits::Converter, BigInt};
// pub use multi_party_eddsa::protocols::aggsig::*;


pub fn r_error_to_c_string(e: failure::Error) -> *mut c_char {
    CString::new(format!("Error: {}", e)).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn add_method(nx: i32, ny: i32) -> i32{
    return nx + ny;
}

#[no_mangle]
pub unsafe extern "C" fn contact_with_str(inx: *const c_char) -> *mut c_char{
    info!("contact_with_str -------- rust!");
    let c_str = unsafe { CStr::from_ptr(inx) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };
    CString::new("这是一个测试-".to_owned() + recipient).unwrap().into_raw()
}


#[no_mangle]
pub extern  "C" fn create_client(endpoint: *const c_char) -> c_longlong{

    let endpoint_c = unsafe { CStr::from_ptr(endpoint) };
    let endpoint_rs = match endpoint_c.to_str() {
        Err(_) => "Endpoint Invalid!",
        Ok(string) => string,
    };

    let client_shim = ClientShim::new(endpoint_rs.to_string(), None);
    
    client_shim.test(1);


    Box::into_raw(Box::new(client_shim)) as c_longlong
}

/**
 * 这里可能会有一些问题
 * 1. iOS/Android/PC端的路径或者沙盒兼容性问题，
 * 2. 根据 1，下面函数中wallet创建的路径都是直接用的FILENAME，这个在PC上没有问题，
 *    代表是的当前路径，但是Android/iOS就有问题了，路径就是个错误的
 * 3. 在Create Wallet后，会有一个保存wallet的操作，但是save方法的存储路径也是默认的
 *    故此也存在 2 中的问题
 * 4. 如果std:fs 能自动适配iOS/Android那么上述问题就不存在
 * 5. createWallet 返回类型是一个wallet的指针值
*/
#[no_mangle]
pub extern "C" fn create_wallet(network: *const c_char, cclient_shim_num_ptr: c_longlong) -> c_longlong{
    let network_c = unsafe { CStr::from_ptr(network) };
    let network_rs = match network_c.to_str() {
        Err(_) => "Endpoint Invalid!",
        Ok(string) => string,
    };

    let cclient_shim_ptr = cclient_shim_num_ptr as *mut ClientShim<reqwest::Client>;

    assert!(!cclient_shim_ptr.is_null(), "cclient_shim_ptr is null!");

    let cclient_shim: &mut ClientShim<reqwest::Client> = unsafe {
        &mut *cclient_shim_ptr
    };
    
    cclient_shim.test(2);

    println!("'create: ");
    println!("Network: [{}], Creating wallet", network_rs);
    let wallet = wallet::Wallet::new(&cclient_shim, &network_rs);
    println!("'create wallet ---------------- 1");
    wallet.save();
    println!("Network: [{}], Wallet saved to disk", &network_rs);

    Box::into_raw(Box::new(wallet)) as c_longlong
}   

#[no_mangle]
pub extern "C" fn drive_new_address_wallet(network: *const c_char,) ->  *mut c_char{

    info!("drive_new_address_wallet -------- rust!");
    let network_c = unsafe { CStr::from_ptr(network) };
    let network_rs = match network_c.to_str() {
        Err(_) => "Endpoint Invalid!",
        Ok(string) => string,
    };

    let mut wallet: wallet::Wallet = wallet::Wallet::load();
    println!("Load wallet: [{}]", wallet.id);
    let address = wallet.get_new_eth_address();
    println!("Network: [{}], Wallet: {} saved to disk", &network_rs, "0x".to_owned() + &address.to_hex());
    wallet.save();

    CString::new(address.to_hex()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn load_wallet() -> c_longlong{
    println!("'load: ");
    let mut wallet: wallet::Wallet = wallet::Wallet::load();

    Box::into_raw(Box::new(wallet)) as c_longlong
}

#[no_mangle]
pub extern "C" fn simple_sign_message(msg: *const c_char, address: *const c_char, network: *const c_char, cclient_shim_num_ptr: c_longlong){
    let mut wallet: wallet::Wallet = wallet::Wallet::load();
    println!("Load wallet: [{}]", wallet.id);
    println!("Sign: ");

    let msg_c = unsafe { CStr::from_ptr(msg) };
    let msg_rs = match msg_c.to_str() {
        Err(_) => "Endpoint Invalid!",
        Ok(string) => string,
    };

    let mut msg_buf = "Test Signature";
    println!("message: [{}]", msg_buf);

    let address_c = unsafe { CStr::from_ptr(address) };
    let address_rs = match address_c.to_str() {
        Err(_) => "ETH Address Invalid!",
        Ok(string) => string,
    };

    let network_c = unsafe { CStr::from_ptr(network) };
    let network_rs = match network_c.to_str() {
        Err(_) => "Endpoint Invalid!",
        Ok(string) => string,
    };

    let cclient_shim_ptr = cclient_shim_num_ptr as *mut ClientShim<reqwest::Client>;

    let cclient_shim: &mut ClientShim<reqwest::Client> = unsafe {
        assert!(!cclient_shim_ptr.is_null());
        &mut *cclient_shim_ptr
    };

    // create a Sha256 object
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(msg_buf);

    // read hash digest and consume hasher
    let e_msg = hasher.finalize();

    wallet.sign(&e_msg, &address_rs, &cclient_shim);
    println!("Network: [{}], MPC signature verified", &network_rs);

}


#[no_mangle]
pub extern "C" fn eth_enter(cclient_shim_num_ptr: c_longlong, address: *const c_char){
    let cclient_shim_ptr = cclient_shim_num_ptr as *mut ClientShim<reqwest::Client>;

    let cclient_shim: &mut ClientShim<reqwest::Client> = unsafe {
        assert!(!cclient_shim_ptr.is_null());
        &mut *cclient_shim_ptr
    };

    let address_c = unsafe { CStr::from_ptr(address) };
    let address_rs = match address_c.to_str() {
        Err(_) => "ETH Address Invalid!",
        Ok(string) => string,
    };

    const RPC_URL: &str = "https://eth.llamarpc.com";
    println!("'derive: ");
    let mut wallet: wallet::Wallet = wallet::Wallet::load();
    println!("Wallet: [{}], loaded", wallet.id);

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { 
        match Provider::<Http>::try_from(RPC_URL) {
            Ok(provider) => {
                let block_number: U64 = provider.get_block_number().await.unwrap();
                println!("{block_number}");
            }
            Err(err) => {
                eprintln!("Failed to create provider: {}", err);
                return;
            }
        }
    });

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
    wallet.sign(&msg, &address_rs, &cclient_shim);
}
