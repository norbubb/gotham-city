// Gotham-city
//
// Copyright 2018 by Kzen Networks (kzencorp.com)
// Gotham city is free software: you can redistribute
// it and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
#![cfg(test)]

use crate::{routes::ecdsa, server};
use std::collections::HashMap;
use std::env;
use std::time::Instant;
use time_test::time_test;

use rocket::{http::ContentType, http::{Header, Status}, local::blocking::Client};
    use two_party_ecdsa::curv::arithmetic::traits::Converter;
    use two_party_ecdsa::curv::cryptographic_primitives::twoparty::dh_key_exchange_variant_with_pok_comm::*;
    use two_party_ecdsa::{party_one, party_two, curv::BigInt};
    use floating_duration::TimeFormat;
    use kms::chain_code::two_party as chain_code;
    use kms::ecdsa::two_party::{MasterKey2, party1};

fn key_gen(client: &Client) -> (String, MasterKey2) {
    time_test!();

    /*************** START: FIRST MESSAGE ***************/
    let start = Instant::now();

    let response = client
        .post("/ecdsa/keygen/first")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 first message",
        TimeFormat(start.elapsed())
    );
    let res_body = response.into_string().unwrap();

    let (id, kg_party_one_first_message): (String, party_one::KeyGenFirstMsg) =
        serde_json::from_str(&res_body).unwrap();

    let start = Instant::now();

    let (kg_party_two_first_message, kg_ec_key_pair_party2) = MasterKey2::key_gen_first_message();

    println!(
        "{} Client: party2 first message",
        TimeFormat(start.elapsed())
    );
    /*************** END: FIRST MESSAGE ***************/

    /*************** START: SECOND MESSAGE ***************/
    let body = serde_json::to_string(&kg_party_two_first_message.d_log_proof).unwrap();

    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/keygen/{}/second", id))
        .body(body)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 second message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let kg_party_one_second_message: party1::KeyGenParty1Message2 =
        serde_json::from_str(&res_body).unwrap();

    let start = Instant::now();

    let key_gen_second_message = MasterKey2::key_gen_second_message(
        &kg_party_one_first_message,
        &kg_party_one_second_message,
    );
    assert!(key_gen_second_message.is_ok());

    println!(
        "{} Client: party2 second message",
        TimeFormat(start.elapsed())
    );

    let (party_two_second_message, party_two_paillier, party_two_pdl_chal) =
        key_gen_second_message.unwrap();

    /*************** END: SECOND MESSAGE ***************/

    /*************** START: THIRD MESSAGE ***************/
    let body = serde_json::to_string(&party_two_second_message.pdl_first_message).unwrap();

    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/keygen/{}/third", id))
        .body(body)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 third message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let party_one_third_message: party_one::PDLFirstMessage =
        serde_json::from_str(&res_body).unwrap();

    let start = Instant::now();

    let pdl_decom_party2 = MasterKey2::key_gen_third_message(&party_two_pdl_chal);

    println!(
        "{} Client: party2 third message",
        TimeFormat(start.elapsed())
    );
    /*************** END: THIRD MESSAGE ***************/

    /*************** START: FOURTH MESSAGE ***************/

    let party_2_pdl_second_message = pdl_decom_party2;
    let request = party_2_pdl_second_message;
    let body = serde_json::to_string(&request).unwrap();

    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/keygen/{}/fourth", id))
        .body(body)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 fourth message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let party_one_pdl_second_message: party_one::PDLSecondMessage =
        serde_json::from_str(&res_body).unwrap();

    let start = Instant::now();

    MasterKey2::key_gen_fourth_message(
        &party_two_pdl_chal,
        &party_one_third_message,
        &party_one_pdl_second_message,
    )
    .expect("pdl error party1");

    println!(
        "{} Client: party2 fourth message",
        TimeFormat(start.elapsed())
    );
    /*************** END: FOURTH MESSAGE ***************/

    /*************** START: CHAINCODE FIRST MESSAGE ***************/
    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/keygen/{}/chaincode/first", id))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 chain code first message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let cc_party_one_first_message: Party1FirstMessage = serde_json::from_str(&res_body).unwrap();

    let start = Instant::now();
    let (cc_party_two_first_message, cc_ec_key_pair2) =
        chain_code::party2::ChainCode2::chain_code_first_message();

    println!(
        "{} Client: party2 chain code first message",
        TimeFormat(start.elapsed())
    );
    /*************** END: CHAINCODE FIRST MESSAGE ***************/

    /*************** START: CHAINCODE SECOND MESSAGE ***************/
    let body = serde_json::to_string(&cc_party_two_first_message.d_log_proof).unwrap();

    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/keygen/{}/chaincode/second", id))
        .body(body)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 chain code second message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let cc_party_one_second_message: Party1SecondMessage = serde_json::from_str(&res_body).unwrap();

    let start = Instant::now();
    let _cc_party_two_second_message = chain_code::party2::ChainCode2::chain_code_second_message(
        &cc_party_one_first_message,
        &cc_party_one_second_message,
    );

    println!(
        "{} Client: party2 chain code second message",
        TimeFormat(start.elapsed())
    );
    /*************** END: CHAINCODE SECOND MESSAGE ***************/

    let start = Instant::now();
    let party2_cc = chain_code::party2::ChainCode2::compute_chain_code(
        &cc_ec_key_pair2,
        &cc_party_one_second_message.comm_witness.public_share,
    )
    .chain_code;

    println!(
        "{} Client: party2 chain code second message",
        TimeFormat(start.elapsed())
    );
    /*************** END: CHAINCODE COMPUTE MESSAGE ***************/

    println!(
        "{} Network/Server: party1 master key",
        TimeFormat(start.elapsed())
    );

    let start = Instant::now();
    let party_two_master_key = MasterKey2::set_master_key(
        &party2_cc,
        &kg_ec_key_pair_party2,
        &kg_party_one_second_message
            .ecdh_second_message
            .comm_witness
            .public_share,
        &party_two_paillier,
    );

    println!("{} Client: party2 master_key", TimeFormat(start.elapsed()));
    /*************** END: MASTER KEYS MESSAGE ***************/

    (id, party_two_master_key)
}

fn sign(
    client: &Client,
    id: String,
    master_key_2: MasterKey2,
    message: BigInt,
) -> party_one::SignatureRecid {
    time_test!();
    let (eph_key_gen_first_message_party_two, eph_comm_witness, eph_ec_key_pair_party2) =
        MasterKey2::sign_first_message();

    let request: party_two::EphKeyGenFirstMsg = eph_key_gen_first_message_party_two;

    let body = serde_json::to_string(&request).unwrap();

    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/sign/{}/first", id))
        .body(body)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 sign first message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let sign_party_one_first_message: party_one::EphKeyGenFirstMsg =
        serde_json::from_str(&res_body).unwrap();

    let x_pos = BigInt::from(0u32);
    let y_pos = BigInt::from(21u32);

    let child_party_two_master_key = master_key_2.get_child(vec![x_pos.clone(), y_pos.clone()]);

    let start = Instant::now();

    let party_two_sign_message = child_party_two_master_key.sign_second_message(
        &eph_ec_key_pair_party2,
        eph_comm_witness.clone(),
        &sign_party_one_first_message,
        &message,
    );

    println!(
        "{} Client: party2 sign second message",
        TimeFormat(start.elapsed())
    );

    let request: ecdsa::SignSecondMsgRequest = ecdsa::SignSecondMsgRequest {
        message,
        party_two_sign_message,
        x_pos_child_key: x_pos,
        y_pos_child_key: y_pos,
    };

    let body = serde_json::to_string(&request).unwrap();

    let start = Instant::now();

    let response = client
        .post(format!("/ecdsa/sign/{}/second", id))
        .body(body)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    println!(
        "{} Network/Server: party1 sign second message",
        TimeFormat(start.elapsed())
    );

    let res_body = response.into_string().unwrap();
    let signature_recid: party_one::SignatureRecid = serde_json::from_str(&res_body).unwrap();

    signature_recid
}

#[test]
fn key_gen_and_sign() {
    // Passthrough mode
    env::set_var("region", "");
    env::set_var("pool_id", "");
    env::set_var("issuer", "");
    env::set_var("audience", "");

    time_test!();

    let settings = HashMap::<String, String>::from([
        ("db".to_string(), "local".to_string()),
        ("db_name".to_string(), "KeyGenAndSign".to_string()),
    ]);
    let client = Client::tracked(server::get_server(settings)).expect("valid rocket instance");

    let (id, master_key_2): (String, MasterKey2) = key_gen(&client);

    let message = BigInt::from(1234u32);

    let signature: party_one::SignatureRecid = sign(&client, id, master_key_2, message);

    println!(
        "s = (r: {}, s: {}, recid: {})",
        signature.r.to_hex(),
        signature.s.to_hex(),
        signature.recid
    );
}

#[test]
fn authentication_test_invalid_token() {
    env::set_var("region", "region");
    env::set_var("pool_id", "pool_id");
    env::set_var("issuer", "issuer");
    env::set_var("audience", "audience");

    let settings = HashMap::<String, String>::from([
        ("db".to_string(), "local".to_string()),
        (
            "db_name".to_string(),
            "AuthenticationTestInvalidToken".to_string(),
        ),
    ]);
    let client = Client::tracked(server::get_server(settings)).expect("valid rocket instance");

    let auth_header = Header::new("Authorization", "Bearer a");
    let response = client
        .post("/ecdsa/keygen/first")
        .header(ContentType::JSON)
        .header(auth_header)
        .dispatch();

    assert_eq!(401, response.status().code);
}

#[test]
fn authentication_test_expired_token() {
    env::set_var("region", "region");
    env::set_var("pool_id", "pool_id");
    env::set_var("issuer", "issuer");
    env::set_var("audience", "audience");

    let settings = HashMap::<String, String>::from([
        ("db".to_string(), "local".to_string()),
        (
            "db_name".to_string(),
            "AuthenticationTestExpiredToken".to_string(),
        ),
    ]);
    let client = Client::tracked(server::get_server(settings)).expect("valid rocket instance");

    let token: String = "Bearer eyJraWQiOiJZeEdoUlhsTytZSWpjU2xWZFdVUFA1dHhWd\
                             FRSTTNmTndNZTN4QzVnXC9YZz0iLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJjNDAz\
                             ZTBlNy1jM2QwLTRhNDUtODI2Mi01MTM5OTIyZjc5NTgiLCJhdWQiOiI0cG1jaXUx\
                             YWhyZjVzdm1nbTFobTVlbGJ1cCIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJjdXN0\
                             b206ZGV2aWNlUEsiOiJbXCItLS0tLUJFR0lOIFBVQkxJQyBLRVktLS0tLVxcbk1G\
                             a3dFd1lIS29aSXpqMENBUVlJS29aSXpqMERBUWNEUWdBRUdDNmQ1SnV6OUNPUVVZ\
                             K08rUUV5Z0xGaGxSOHpcXHJsVjRRTTV1ZUhsQjVOTVQ2dm04c1dFMWtpak5udnpP\
                             WDl0cFRZUEVpTEIzbHZORWNuUmszTXRRZVNRPT1cXG4tLS0tLUVORCBQVUJMSUMg\
                             S0VZLS0tLS1cIl0iLCJ0b2tlbl91c2UiOiJpZCIsImF1dGhfdGltZSI6MTU0NjUz\
                             MzM2NywiaXNzIjoiaHR0cHM6XC9cL2NvZ25pdG8taWRwLnVzLXdlc3QtMi5hbWF6\
                             b25hd3MuY29tXC91cy13ZXN0LTJfZzlqU2xFYUNHIiwiY29nbml0bzp1c2VybmFt\
                             ZSI6ImM0MDNlMGU3LWMzZDAtNGE0NS04MjYyLTUxMzk5MjJmNzk1OCIsImV4cCI6\
                             MTU0NzEwNzI0OSwiaWF0IjoxNTQ3MTAzNjQ5LCJlbWFpbCI6ImdhcnkrNzgyODJA\
                             a3plbmNvcnAuY29tIn0.WLo9fiDiovRqC1RjR959aD8O1E3lqi5Iwnsq4zobqPU5\
                             yZHW2FFIDwnEGf3UmQWMLgscKcuy0-NoupMUCbTvG52n5sPvOrCyeIpY5RkOk3mH\
                             enH3H6jcNRA7UhDQwhMu_95du3I1YHOA173sPqQQvmWwYbA8TtyNAKOq9k0QEOuq\
                             PWRBXldmmp9pxivbEYixWaIRtsJxpK02ODtOUR67o4RVeVLfthQMR4wiANO_hKLH\
                             rt76DEkAntM0KIFODS6o6PBZw2IP4P7x21IgcDrTO3yotcc-RVEq0X1N3wI8clr8\
                             DaVVZgolenGlERVMfD5i0YWIM1j7GgQ1fuQ8J_LYiQ"
        .to_string();

    let auth_header = Header::new("Authorization", token);

    let response = client
        .post("/ecdsa/keygen/first")
        .header(ContentType::JSON)
        .header(auth_header)
        .dispatch();

    assert_eq!(401, response.status().code);
}
