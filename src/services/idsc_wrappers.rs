use std::sync::Arc;
use ethers::abi::Uint;
use crate::{EthClient, LocalContractInstance};

/// Retrieves the first free VC ID from the IDSC
pub async fn get_free_vc_id(
    idsc_instance: LocalContractInstance,
    eth_client: Arc<EthClient>
) -> Uint {    
    idsc_instance
        .connect(eth_client.clone())
        .method::<_, Uint>("getFreeVCid", ())
        .unwrap()
        .call()
        .await.unwrap()
}