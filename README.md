### Docker compose example
```yaml
services:
  sedimark-issuer-rs:
    build:
      context: .
      dockerfile: Dockerfile
    image: sedimark-issuer-rs
    hostname: sedimark-issuer-rs
    container_name: sedimark-issuer-rs
    restart: unless-stopped
    volumes:
      - ./docker_data:/data
    ports:
      - "3213:3213"
    depends_on:
      sedimark-issuer-postgres:
       condition: service_healthy 
    networks:
      - dlt-booth-net
    logging:
      driver: "local"
    environment:
      # Rust flags
      RUST_BACKTRACE: 1
      RUST_LOG: debug
      # HTTP SERVER SETUP
      HOST_ADDRESS: 0.0.0.0
      HOST_PORT: 3213
      # DLT CONFIG
      NODE_URL: https://stardust.unican.sedimark.eu
      FAUCET_API_ENDPOINT: https://faucet.tangle.stardust.linksfoundation.com/api/enqueue
      RPC_PROVIDER: https://stardust.unican.sedimark.eu/sedimark-chain
      CHAIN_ID: 1074
      ISSUER_URL: http://sedimark-issuer-rs:3213/api/credentials/
      # KEY STORAGE CONFIGURATION
      KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH: ./key_storage.stronghold
      KEY_STORAGE_STRONGHOLD_PASSWORD: some_hopefully_secure_password
      KEY_STORAGE_MNEMONIC: strategy exercise globe absent hill help demand mistake rival report fame owner drift treat gather gospel anxiety limb tribe exhaust october foil title account
      # ISSUER CONFIG
      ISSUER_PRIVATE_KEY: issuer_private key
      IDENTITY_SC_ADDRESS: sc_address
      # DATABASE CONNECTION CONFIG
      DB_USER: postgres
      DB_PASSWORD: issuer
      DB_NAME: identity
      DB_HOST: sedimark-issuer-postgres
      DB_PORT: 5432
      DB_MAX_POOL_SIZE: 16
  sedimark-issuer-postgres:
    container_name: sedimark-issuer-postgres
    hostname: sedimark-issuer-postgres
    image: postgres:16
    ports:
      - "5433:5432"
    volumes: 
      - ./server/postgresdata:/var/lib/postgresql/data
      - ./server/src/repository/sql/dbinit.sql:/docker-entrypoint-initdb.d/dbinit.sql
    restart: always
    healthcheck:
      test: [ "CMD-SHELL", "pg_isready -d $${POSTGRES_DB} -U $${POSTGRES_USER}" ]
      interval: 10s
      timeout: 5s
      retries: 5
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: issuer
      POSTGRES_DB: identity
    networks:
      - dlt-booth-net
    logging:
      driver: "local"

networks:
  dlt-booth-net:
    external: true
```

### Kubernetes deployment 

To deploy the issuer in a Kubernetes cluster, first set the necessary environment variables to be parsed in the manifests:

| Name                                   | Description                                   | Example (in clear text)                                                                      | base64 |
|----------------------------------------|-----------------------------------------------|--------------------------------------------------------------------------------------------|--------|
| ISSUER_NAMESPACE                       | Kubernetes namespace for the issuer           | dlt-booth                                                                                  | No     |
| ISSUER_APP_NAME                        | Application name used for Kubernetes resources| issuer                                                                                     | No     |
| STORAGECLASS                           | Kubernetes storage class for persistent volumes| nfs-storageclass                                                                          | No     |
| ISSUER_NODE_URL                        | URL for the blockchain node                   | https://stardust.unican.sedimark.eu                                                        | No     |
| ISSUER_FAUCET_API_ENDPOINT             | API endpoint for the blockchain faucet        | https://faucet.tangle.stardust.linksfoundation.com/api/enqueue                             | No     |
| ISSUER_RPC_PROVIDER                    | RPC provider URL for blockchain               | https://stardust.unican.sedimark.eu/sedimark-chain                                         | No     |
| ISSUER_CHAIN_ID                        | Blockchain network ID                         | 1111                                                                                       | No     |
| ISSUER_DB_USER                         | Database username                             | postgres                                                                                   | No     |
| ISSUER_DB_PASSWORD                     | Database password                             | issuer                                                                                     | Yes    |
| ISSUER_DOCKER_REGISTRY_CREDENTIALS     | Base64 encoded Docker registry credentials    | {"auths":{"registry.example.com":{"username":"user","password":"pass"}}}                   | Yes    |
| ISSUER_KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH | Path to the Stronghold snapshot file     | ./key_storage.stronghold                                                                   | Yes    |
| ISSUER_KEY_STORAGE_STRONGHOLD_PASSWORD | Password for the Stronghold snapshot          | some_hopefully_secure_password                                                             | Yes    |
| ISSUER_KEY_STORAGE_MNEMONIC            | Mnemonic for key recovery                     | plastic volcano debate cruel wisdom jacket survey voyage panic lecture uniform forest sketch fiber alcohol symbol museum rainbow orbit garden laptop autumn exact melody | Yes    |
| ISSUER_PRIVATE_KEY                     | Private key for the issuer                    | issuer_private_key                                                                         | Yes    |
| ISSUER_IDENTITY_SC_ADDRESS             | Smart contract address for identity           | sc_address                                                                                 | Yes    |
| ISSUER_DOCKER_IMAGE                    | Docker image name                             | registry.example.com/sedimark-issuer-rs                                                    | No     |
| ISSUER_IMAGETAG                        | Docker image tag for the issuer               | latest                                                                                     | No     |
| ISSUER_POSTGRES_IMAGETAG               | Docker image tag for PostgreSQL               | 16                                                                                         | No     |

Then, apply the Kubernetes manifests:

```bash 
cat ./kubernetes/*.yaml | envsubst | kubectl apply -f -
```

The manifests don't provide any ingress, so to access the issuer, you can use port-forwarding:

```bash 
kubectl port-forward -n $ISSUER_NAMESPACE svc/$ISSUER_APP_NAME 3213:3213
```
