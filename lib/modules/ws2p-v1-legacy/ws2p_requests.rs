use duniter_network::requests::OldNetworkRequest;

pub fn network_request_to_json(request: &OldNetworkRequest) -> serde_json::Value {
    let (request_id, request_type, request_params) = match *request {
        OldNetworkRequest::GetCurrent(ref req_full_id, _receiver) => {
            (req_full_id.1, "CURRENT", json!({}))
        }
        OldNetworkRequest::GetBlocks(ref req_full_id, _receiver, count, from_mumber) => (
            req_full_id.1,
            "BLOCKS_CHUNK",
            json!({
                "count": count,
                "fromNumber": from_mumber
            }),
        ),
        OldNetworkRequest::GetRequirementsPending(ref req_full_id, _receiver, min_cert) => (
            req_full_id.1,
            "WOT_REQUIREMENTS_OF_PENDING",
            json!({ "minCert": min_cert }),
        ),
        OldNetworkRequest::GetConsensus(_) => {
            panic!("GetConsensus() request must be not convert to json !");
        }
        OldNetworkRequest::GetHeadsCache(_) => {
            panic!("GetHeadsCache() request must be not convert to json !");
        }
        OldNetworkRequest::GetEndpoints(_) => {
            panic!("GetEndpoints() request must be not convert to json !");
        }
    };

    json!({
        "reqId": request_id,
        "body" : {
            "name": request_type,
            "params": request_params
        }
    })
}
