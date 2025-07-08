use alloy::sol;

sol!(
    #[sol(rpc)]
    Identity,
    "../smart-contracts/Identity.json"
);