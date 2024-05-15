// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use deadpool_postgres::Pool;
use ethers::types::{Signature, Address};
use std::{str::FromStr, sync::Arc};
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::{credential::{Jws, Jwt}, document::verifiable::JwsVerificationOptions, iota::{IotaIdentityClientExt, IotaDID}};
use iota_sdk::U256;

use crate::{contracts::identity::Identity, repository::operations::HoldersRequestsExt, services::issuer_vc::create_credential, utils::{eth::SignerMiddlewareShort, iota::IotaState}};
use crate::errors::IssuerError;

use crate::{
    dtos::identity_dtos::CredentialRequestDTO, 
    services::issuer_vc::update_identity_sc
};

pub async fn create_credential_service(
    pool: Pool,
    signer: &Arc<SignerMiddlewareShort>,
    iota_state: &IotaState,
    request_dto: CredentialRequestDTO
) -> Result<(U256,Jwt), IssuerError>  {
    
    let pg_client = &pool.get().await?;
    // read the request from the DB 
    let holder_request = pg_client.get_holder_request(&request_dto.did).await?;
    log::info!("{:?}", holder_request);
    // first check request is not empty //TODO: for me this is useless
    // is_empty_request(holder_request.clone()).then(|| 0).ok_or(IssuerError::NonExistingRequestError)?; // fix, should return error if is empty

    // resolve DID Doc and extract public key
    let holder_document = iota_state.client.resolve_did(&IotaDID::parse(holder_request.did)?).await?;
    
    // Verify DID ownership, i.e. challenge signed equal to the stored nonce (anti replay)
    let _decoded_jws =  holder_document.verify_jws(
        &Jws::from(request_dto.identity_signature), // TODO: evaluate usage of auth header
        None,
        &EdDSAJwsVerifier::default(),
        &JwsVerificationOptions::default().nonce(&holder_request.nonce),
    )?;

    // Get the first free credential id from Identity Smart Contract
    let identity_addr: Address = std::env::var("IDENTITY_SC_ADDRESS").expect("$IDENTITY_SC_ADDRESS must be set").parse().map_err(|_| IssuerError::ContractAddressRecoveryError)?;
    let identity_sc = Identity::new(identity_addr, signer.into());
    
    let credential_id = identity_sc.get_free_v_cid().call().await.map_err(|err| IssuerError::ContractError(err.to_string()))?;

    // Create and sign the credential
    let (credential_jwt, decoded_jwt_credential) = create_credential(
        &holder_document,
        &iota_state.issuer_document, 
        credential_id, 
        &iota_state.key_storage,
        &iota_state.issuer_identity.fragment,
        request_dto.credential_subject
    ).await.map_err(|_| IssuerError::OtherError("Conversion error".to_owned()))?;

    // Verify the EOA ownership
    log::info!("Wallet sign: {:?}", request_dto.wallet_signature);
    let wallet_sign = Signature::from_str(request_dto.wallet_signature.as_str())?;
    log::info!("signature {:?}", wallet_sign);
    let vm = holder_document.resolve_method("#ethAddress", None).ok_or(IssuerError::EthMethodNotFound)?;

    vm.type_().to_string().eq("EcdsaSecp256k1RecoverySignature2020").then(|| Some(())).ok_or(IssuerError::InvalidVerificationMethodType)?;
    let eth_addr = vm.properties()
    .get("blockchainAccountId")
    .ok_or(IssuerError::InvalidVerificationMethodType)?
    .as_str().ok_or(IssuerError::InvalidVerificationMethodType)?
    .strip_prefix("eip155:1:")
    .ok_or(IssuerError::InvalidVerificationMethodType)?;

    log::info!("eth addr: {}", eth_addr);
    let address: Address = eth_addr.parse().map_err(|_| IssuerError::AddressRecoveryError)?;

    wallet_sign.verify(holder_request.nonce.clone(), address)?;
    log::info!("Wallet signature verification success!");
    
    // Update Identity SC, addUser 
    update_identity_sc(
        identity_sc, 
        decoded_jwt_credential,
        credential_id, 
        holder_request.nonce,
        &request_dto.wallet_signature, 
    ).await?;

    pg_client.remove_holder_request(&holder_document.id().to_string()).await?;

    Ok((credential_id,credential_jwt))
}