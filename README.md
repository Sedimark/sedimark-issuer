# mediterraneus-issuer-rs

![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![Iota](https://img.shields.io/badge/iota-29334C?style=for-the-badge&logo=iota&logoColor=white)

Issuer of verifiable credentials using smart contracts to bind the pseudonymous identity with the Self Sovereign Identity. Sample implementation for the Mediterraneus Protocol.

## Issuer initialization
The issuer must posses an SSI comprising of at least a DID. At application start up the issuer creates a new identity or retrieves it from the local database. 
This is an insecure implementation due to the clear-text storage of the sensitive information of its identity. This must be solved with the usage of 
secure storage like Stronghold.

However, the stringhold binding are not aligned with the current status of the identity framework. Hence, as soon as they will updated and published, this solution must
be updated.

## Verifiable Credential Issuance
Before issuing a VC the Issuer must perform the following operations:

1. Resolve the requester's DID and retrieve the verification method public key.

## Running the Application

https://github.com/actix/examples/blob/master/databases/postgres/src/main.rs