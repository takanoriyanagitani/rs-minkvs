use core::net::SocketAddr;

use rs_minkvs::minkvs::v1::min_kvs_service_server::MinKvsServiceServer;

use rs_minkvs::basic::btree::KvsBtree;

pub const FILE_DESC_SET: &[u8] = tonic::include_file_descriptor_set!("minkvs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let refl_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESC_SET)
        .build_v1()?;

    let addr: SocketAddr = std::env::var("ENV_LISTEN_ADDR")
        .ok()
        .unwrap_or("127.0.0.1:50051".into())
        .parse()?;

    let kvsbt = KvsBtree::default();
    let kvs3: MinKvsServiceServer<_> = MinKvsServiceServer::new(kvsbt);

    tonic::transport::Server::builder()
        .add_service(kvs3)
        .add_service(refl_svc)
        .serve(addr)
        .await?;

    Ok(())
}
