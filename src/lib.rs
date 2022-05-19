pub mod bencode;
pub mod metainfo;
pub mod tracker;
use hex::FromHex;
use log::*;
use tracker::Event;
use tracker::RequestParameters;
use tracker::*;

fn init_request_parameters(info_hash: Vec<u8>, peer_id: Vec<u8>) -> RequestParameters {
    // TODO: Should check what values to put on left field
    RequestParameters {
        info_hash,
        peer_id,
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: 0,
        event: Event::Started,
    }
}

pub fn run() -> Result<(), tracker::TrackerError> {
    pretty_env_logger::init();
    info!("hola");
    let info_hash = <[u8; 20]>::from_hex("2c6b6858d61da9543d4231a71db4b1c9264b0685").unwrap();
    let peer_id = info_hash;
    let params: RequestParameters = init_request_parameters(info_hash.to_vec(), peer_id.to_vec());
    let _res = get_peer_list(params)?;
    info!("tracker response successfully parsed");
    Ok(())
}
