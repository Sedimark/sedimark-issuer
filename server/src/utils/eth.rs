use ethers::providers::{Provider, Http};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::SignerMiddleware;
use ethers::prelude::Wallet;

pub type SignerMiddlewareShort = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;