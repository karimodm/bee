// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_WHITE_FLAG,
        filters::{with_storage, with_tangle},
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::WhiteFlagResponse},
};

use bee_ledger::consensus::metadata::WhiteFlagMetadata;
use bee_message::{milestone::MilestoneIndex, MessageId};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde_json::Value as JsonValue;
use warp::{reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("whiteflag")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::post())
        .and(has_permission(ROUTE_WHITE_FLAG, public_routes, allowed_ips))
        .and(warp::body::json())
        .and(with_storage(storage))
        .and(with_tangle(tangle))
        .and_then(white_flag)
}

pub(crate) async fn white_flag<B: StorageBackend>(
    body: JsonValue,
    _storage: ResourceHandle<B>,
    _tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    let index_json = &body["index"];
    let parents_json = &body["parentMessageIds"];

    let index = if index_json.is_null() {
        return Err(reject::custom(CustomRejection::BadRequest(
            "Invalid index: expected a MilestoneIndex".to_string(),
        )));
    } else {
        MilestoneIndex(
            index_json
                .as_str()
                .ok_or_else(|| {
                    reject::custom(CustomRejection::BadRequest(
                        "Invalid index: expected a MilestoneIndex".to_string(),
                    ))
                })?
                .parse::<u32>()
                .map_err(|_| {
                    reject::custom(CustomRejection::BadRequest(
                        "Invalid index: expected a MilestoneIndex".to_string(),
                    ))
                })?,
        )
    };

    let parents: Vec<MessageId> = if parents_json.is_null() {
        return Err(reject::custom(CustomRejection::BadRequest(
            "Invalid parents: expected an array of MessageId".to_string(),
        )));
    } else {
        let array = parents_json.as_array().ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "Invalid parents: expected an array of MessageId".to_string(),
            ))
        })?;
        let mut message_ids = Vec::new();
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    reject::custom(CustomRejection::BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    ))
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(CustomRejection::BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    ))
                })?;
            message_ids.push(message_id);
        }
        message_ids
    };

    let mut _metadata = WhiteFlagMetadata::new(index);

    println!("{:?}", index);
    println!("{:?}", parents);

    Ok(warp::reply::json(&SuccessBody::new(WhiteFlagResponse {
        merkle_tree_hash: String::from("Bee"),
    })))
}
