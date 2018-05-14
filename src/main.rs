extern crate duniter_core;
extern crate duniter_tui;
extern crate duniter_ws2p;

use duniter_core::DuniterCore;
use duniter_tui::TuiModule;
use duniter_ws2p::WS2PModule;

fn main() {
    // Get software name and version
    let soft_name = env!("CARGO_PKG_NAME");
    let soft_version = env!("CARGO_PKG_VERSION");

    // Run duniter core
    if let Some(mut duniter_core) = DuniterCore::new(soft_name, soft_version) {
        duniter_core.plug::<WS2PModule>();
        duniter_core.plug::<TuiModule>();
        //duniter_core.plug::<PoolModule>();
        //duniter_core.plug::<PowModule>();
        //duniter_core.plug::<GvaModule>();
        //duniter_core.plug::<DasaModule>();
        //duniter_core.plug::<GuiModule>();
        duniter_core.start_blockchain();
    };
}
