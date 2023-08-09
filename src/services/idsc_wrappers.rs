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

pub async fn register_new_vc_idsc(
    idsc_instance: LocalContractInstance,
    eth_client: Arc<EthClient>,
    vc_id: i64, pseudo_sign: String, holder_did: String, exp_unix: i64, issuance_unix: i64, vc_hash: String
) {

    let call = idsc_instance
        .connect(eth_client.clone())
        .method::<(i64, std::string::String, std::string::String, i64, i64, std::string::String), String>(
            "validate_and_store_VC",
            (vc_id, pseudo_sign, holder_did, exp_unix, issuance_unix, vc_hash)
        ).unwrap();
    let pending_tx = call.send().await.unwrap();
    // `await`ing on the pending transaction resolves to a transaction receipt
    pending_tx.confirmations(1).await.unwrap();
}