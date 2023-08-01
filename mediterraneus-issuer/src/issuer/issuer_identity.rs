use anyhow::Ok;

use identity_iota::crypto::KeyPair;
use identity_iota::crypto::KeyType;
use identity_iota::iota::IotaDocument;
use identity_iota::iota::IotaIdentityClientExt;
use identity_iota::prelude::IotaClientExt;
use identity_iota::verification::MethodScope;
use identity_iota::verification::VerificationMethod;
use iota_client::Client;
use iota_client::block::address::Address;
use iota_client::block::output::AliasOutput;
use iota_client::secret::SecretManager;

pub async fn create_identity(client: &Client, wallet_address: Address, secret_manager: &mut SecretManager) -> anyhow::Result<()> {
    // Get the Bech32 human-readable part (HRP) of the network.
    let network_name = client.network_name().await?;

    // Create a new DID document with a placeholder DID.
    // The DID will be derived from the Alias Id of the Alias Output after publishing.
    let mut document: IotaDocument = IotaDocument::new(&network_name);
    
    // Insert a new Ed25519 verification method in the DID document.
    let keypair: KeyPair = KeyPair::new(KeyType::Ed25519)?;
    let method: VerificationMethod = VerificationMethod::new(document.id().clone(), keypair.type_(), keypair.public(), "#key-1")?;
    document.insert_method(method, MethodScope::VerificationMethod)?;

    // Construct an Alias Output containing the DID document, with the wallet address
    // set as both the state controller and governor.
    let alias_output: AliasOutput = client.new_did_output(wallet_address, document, None).await?;

    // Publish the Alias Output and get the published DID document.
    let document: IotaDocument = client.publish_did_output(secret_manager, alias_output).await?;
    println!("Published DID document: {document:#}");

    Ok(())
}  