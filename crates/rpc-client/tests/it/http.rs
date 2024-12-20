use atoms_node_bindings::Anvil;
use atoms_rpc_client::{ClientBuilder, RpcCall};
use base_primitives::U64;

#[tokio::test]
async fn it_makes_a_request() {
    let anvil = Anvil::new().spawn();
    let url = anvil.endpoint();
    let client = ClientBuilder::default().http(url.parse().unwrap());
    let req: RpcCall<_, (), U64> = client.request("xcb_blockNumber", ());
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(2), req);
    let res = timeout.await.unwrap().unwrap();
    assert_eq!(res.to::<u64>(), 0);
}
