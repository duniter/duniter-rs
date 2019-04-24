use dup_crypto::keys::*;
use durs_network_documents::network_endpoint::{EndpointV1, NetworkEndpointApi};
use sqlite::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EndpointApi {
    WS2P,
    //WS2PS,
    //WS2PTOR,
    //DASA,
    //BMA,
    //BMAS,
}

impl From<u32> for EndpointApi {
    fn from(integer: u32) -> Self {
        match integer {
            _ => EndpointApi::WS2P,
        }
    }
}

pub fn string_to_api(api: &str) -> Option<EndpointApi> {
    match api {
        "WS2P" => Some(EndpointApi::WS2P),
        //"WS2PS" => Some(EndpointApi::WS2PS),
        //"WS2PTOR" => Some(EndpointApi::WS2PTOR),
        //"DASA" => Some(EndpointApi::DASA),
        //"BASIC_MERKLED_API" => Some(EndpointApi::BMA),
        //"BMAS" => Some(EndpointApi::BMAS),
        &_ => None,
    }
}

pub fn api_to_integer(api: &NetworkEndpointApi) -> i64 {
    match api.0.as_str() {
        "WS2P" => 1,
        //EndpointApi::WS2PS => 2,
        //EndpointApi::WS2PTOR => 3,
        //EndpointApi::DASA => 4,
        //EndpointApi::BMA => 5,
        //EndpointApi::BMAS => 6,
        _ => 0,
    }
}

pub fn get_endpoints_for_api(db: &Connection, api: &NetworkEndpointApi) -> Vec<EndpointV1> {
    let mut cursor: Cursor = db
        .prepare("SELECT hash_full_id, status, node_id, pubkey, api, version, endpoint, last_check FROM endpoints WHERE api=? ORDER BY status DESC;")
        .expect("get_endpoints_for_api() : Error in SQL request !")
        .cursor();

    cursor
        .bind(&[Value::Integer(api_to_integer(&api))])
        .expect("get_endpoints_for_api() : Error in cursor binding !");
    let mut endpoints = Vec::new();
    while let Some(row) = cursor
        .next()
        .expect("get_endpoints_for_api() : Error in cursor.next()")
    {
        let raw_ep = row[6].as_string().unwrap().to_string();
        let ep_issuer =
            PubKey::Ed25519(ed25519::PublicKey::from_base58(row[3].as_string().unwrap()).unwrap());
        let mut ep = match EndpointV1::parse_from_raw(
            &raw_ep,
            ep_issuer,
            row[1].as_integer().unwrap() as u32,
            row[7].as_integer().unwrap() as u64,
        ) {
            Ok(ep) => ep,
            Err(e) => panic!(format!(
                "Fail to parse endpoint : {} (Error: {:?})",
                raw_ep, e
            )),
        };
        ep.status = row[1].as_integer().unwrap() as u32;
        ep.last_check = row[7].as_integer().unwrap() as u64;

        endpoints.push(ep);
    }
    endpoints
}

pub fn write_endpoint(
    db: &Connection,
    endpoint: &EndpointV1,
    new_status: u32,
    new_last_check: u64,
) {
    let hash_full_id = endpoint
        .node_full_id()
        .expect("Fail to write endpoint : node_full_id() return None !")
        .sha256();
    // Check if endpoint it's already written
    let mut cursor: Cursor = db
        .prepare("SELECT status FROM endpoints WHERE hash_full_id=? ORDER BY status DESC;")
        .expect("write_endpoint() : Error in SQL request !")
        .cursor();
    cursor
        .bind(&[Value::String(hash_full_id.to_string())])
        .expect("write_endpoint() : Error in cursor binding !");

    // If endpoint it's already written, update status
    if let Some(row) = cursor
        .next()
        .expect("write_endpoint() : Error in cursor.next()")
    {
        if row[0].as_integer().expect("fail to read ep status !") as u32 != endpoint.status {
            db.execute(format!(
                "UPDATE endpoints SET status={} WHERE hash_full_id='{}'",
                endpoint.status, hash_full_id
            ))
            .expect("Fail to parse SQL request update endpoint  status !");
        }
    } else {
        let ep_v10 = endpoint;
        db
                    .execute(
                        format!(
                            "INSERT INTO endpoints (hash_full_id, status, node_id, pubkey, api, version, endpoint, last_check) VALUES ('{}', {}, {}, '{}', {}, {}, '{}', {});",
                            ep_v10.hash_full_id.expect("ep_v10.hash_full_id = None"), new_status, ep_v10.node_id.expect("ep_v10.node_id = None").0,
                            ep_v10.issuer.to_string(), api_to_integer(&ep_v10.api),
                            1, ep_v10.raw_endpoint, new_last_check
                        )
                    )
                    .expect("Fail to parse SQL request INSERT endpoint !");
    }
}
