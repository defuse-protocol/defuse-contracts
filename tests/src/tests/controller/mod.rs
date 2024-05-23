use crate::utils::Sandbox;

#[tokio::test]
async fn test_deploy_contract() {
    let sandbox = Sandbox::new().await.unwrap();
    let controller = sandbox.deploy_controller_contract().await;

    assert_eq!(controller.id().as_str(), "controller.test.near");
}
