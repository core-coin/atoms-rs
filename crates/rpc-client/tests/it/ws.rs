use atoms_rpc_client::{ClientBuilder, RpcCall};
use atoms_transport_ws::WsConnect;
use atoms_node_bindings::Anvil;
use base_primitives::U64;

#[tokio::test]
async fn it_makes_a_request() {
    let anvil = Anvil::new().spawn();
    let url = anvil.ws_endpoint();
    let connector = WsConnect { url: url.parse().unwrap(), auth: None };
    let client = ClientBuilder::default().pubsub(connector).await.unwrap();
    let req: RpcCall<_, (), U64> = client.request("xcb_blockNumber", ());
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(15), req);
    let res = timeout.await.unwrap().unwrap();
    assert_eq!(res.to::<u64>(), 0);
}
