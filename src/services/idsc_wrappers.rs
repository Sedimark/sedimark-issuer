use std::sync::Arc;
use ethers::types::U256;
use crate::{EthClient, LocalContractInstance};

/// Retrieves the first free VC ID from the IDSC
pub async fn get_free_vc_id(
    idsc_instance: LocalContractInstance,
    eth_client: Arc<EthClient>
) -> U256 {    
    idsc_instance
        .connect(eth_client.clone())
        .method::<_, U256>("getFreeVCid", ())
        .unwrap()
        .call()
        .await.unwrap()
}