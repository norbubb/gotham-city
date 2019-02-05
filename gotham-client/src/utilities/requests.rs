// Gotham-city
//
// Copyright 2018 by Kzen Networks (kzencorp.com)
// Gotham city is free software: you can redistribute
// it and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//

use super::super::api;
use serde;
use time::PreciseTime;

pub fn post(client_shim: &api::ClientShim, path: &str) -> Option<String> {
    let start = PreciseTime::now();

    let res = client_shim
        .client
        .post(&format!("{}/{}", client_shim.endpoint, path))
        .json("{}")
        .send();

    let end = PreciseTime::now();

    info!("(req {}, took: {})", path, start.to(end));

    Some(res.unwrap().text().unwrap())
}

pub fn postb<T>(client_shim: &api::ClientShim, path: &str, body: T) -> Option<String>
where
    T: serde::ser::Serialize,
{
    let start = PreciseTime::now();

    let res = client_shim
        .client
        .post(&format!("{}/{}", client_shim.endpoint, path))
        .json(&body)
        .send();

    let end = PreciseTime::now();

    info!("(req {}, took: {})", path, start.to(end));

    Some(res.unwrap().text().unwrap())
}
