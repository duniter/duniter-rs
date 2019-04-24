use durs_network_documents::network_head::*;

use std::ops::Deref;

pub fn serialize_head(head: NetworkHead) -> serde_json::Value {
    match head {
        NetworkHead::V2(box_head_v2) => {
            let head_v2 = box_head_v2.deref();
            json!({
                "message": head_v2.message.to_string(),
                "sig": head_v2.sig.to_string(),
                "messageV2": head_v2.message_v2.to_string(),
                "sigV2": head_v2.sig_v2.to_string(),
                "step": head_v2.step + 1
            })
        }
        _ => panic!("HEAD version not supported !"),
    }
}
