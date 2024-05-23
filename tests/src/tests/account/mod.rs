use crate::utils::Sandbox;
use near_sdk::NearToken;

#[tokio::test]
async fn test_deploy_contract() {
    let sandbox = Sandbox::new().await.unwrap();
    let account = sandbox.deploy_account_contract().await;

    assert_eq!(account.id().as_str(), "account.test.near");
    // Some near was spent on the contract deployment.
    assert!(sandbox.balance(account.id()).await < NearToken::from_near(10).as_yoctonear());
}
