use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;

pub mod rpc_client {
    pub use solana_client::nonblocking::rpc_client::RpcClient;
}

pub fn create_rpc_client(endpoint: String) -> RpcClient {
    //RpcClient::new(endpoint)
     rpc_client::RpcClient::new(endpoint)
}

pub async fn get_current_epoch(rpc_client: Arc<RpcClient>) -> u64 {
    // Get the current epoch
    //let epoch_info = rpc_client.get_epoch_info().await.unwrap();
    let epoch_info = rpc_client.get_epoch_info().await.unwrap();
    println!("Epoch info: {:?}", epoch_info);
    epoch_info.epoch
}