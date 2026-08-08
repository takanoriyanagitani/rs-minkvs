use std::sync::Arc;

use tonic::Status;

use tonic::async_trait;
use tonic::{Request, Response};

use tonic::transport::Channel;

use crate::minkvs::v1::min_kvs_service_client::MinKvsServiceClient;

use crate::minkvs::v1::min_kvs_service_server::MinKvsService;

use crate::minkvs::v1::{CountRequest, CountResponse};
use crate::minkvs::v1::{DelRequest, DelResponse};
use crate::minkvs::v1::{ExistsRequest, ExistsResponse};
use crate::minkvs::v1::{GetKeysRequest, GetKeysResponse};
use crate::minkvs::v1::{GetRequest, GetResponse};
use crate::minkvs::v1::{KeyValPair as ProtoKeyValPair, MultiGetRequest, MultiGetResponse};
use crate::minkvs::v1::{MultiSetRequest, MultiSetResponse};
use crate::minkvs::v1::{SetRequest, SetResponse};

pub struct KeyValPair(pub Vec<u8>, pub Vec<u8>);

#[async_trait]
pub trait Kvs: Send + Sync + 'static {
    /// Try gets the value by the key.
    async fn get(&self, key: Vec<u8>) -> Result<Vec<u8>, Status>;

    /// Try gets all keys.
    async fn get_keys(&self, max_keys: u32) -> Result<Vec<Vec<u8>>, Status>;

    /// Try gets the values by the keys.
    async fn multi_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<KeyValPair>>, Status>;

    /// Try set the pair and returns if overwritten or not.
    async fn set(&self, key: Vec<u8>, val: Vec<u8>) -> Result<bool, Status>;

    /// Try delete the pair by the key and returns if it was absent or not.
    async fn del(&self, key: Vec<u8>) -> Result<bool, Status>;

    /// Set multiple pairs and return the number of keys that were overwritten.
    async fn multi_set(&self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<u32, Status>;

    /// Check if a key exists in the store.
    async fn exists(&self, key: Vec<u8>) -> Result<bool, Status>;

    /// Get the total number of keys in the store.
    async fn count(&self) -> Result<u64, Status>;
}

#[async_trait]
impl<T> Kvs for Arc<T>
where
    T: Kvs,
{
    async fn get(&self, key: Vec<u8>) -> Result<Vec<u8>, Status> {
        Kvs::get(self.as_ref(), key).await
    }

    async fn get_keys(&self, max_keys: u32) -> Result<Vec<Vec<u8>>, Status> {
        Kvs::get_keys(self.as_ref(), max_keys).await
    }

    async fn multi_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<KeyValPair>>, Status> {
        Kvs::multi_get(self.as_ref(), keys).await
    }

    async fn set(&self, key: Vec<u8>, val: Vec<u8>) -> Result<bool, Status> {
        Kvs::set(self.as_ref(), key, val).await
    }

    async fn del(&self, key: Vec<u8>) -> Result<bool, Status> {
        Kvs::del(self.as_ref(), key).await
    }

    async fn multi_set(&self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<u32, Status> {
        Kvs::multi_set(self.as_ref(), pairs).await
    }

    async fn exists(&self, key: Vec<u8>) -> Result<bool, Status> {
        Kvs::exists(self.as_ref(), key).await
    }

    async fn count(&self) -> Result<u64, Status> {
        Kvs::count(self.as_ref()).await
    }
}

#[async_trait]
impl<T> MinKvsService for T
where
    T: Kvs,
{
    async fn get(&self, req: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let request: GetRequest = req.into_inner();
        let val: Vec<u8> = self.get(request.key).await?;
        Ok(Response::new(GetResponse { val }))
    }

    async fn get_keys(
        &self,
        req: Request<GetKeysRequest>,
    ) -> Result<Response<GetKeysResponse>, Status> {
        let request: GetKeysRequest = req.into_inner();
        let keys: Vec<Vec<u8>> = self.get_keys(request.max_num_keys).await?;
        Ok(Response::new(GetKeysResponse { keys }))
    }

    async fn multi_get(
        &self,
        req: Request<MultiGetRequest>,
    ) -> Result<Response<MultiGetResponse>, Status> {
        let request: MultiGetRequest = req.into_inner();
        let results: Vec<Option<KeyValPair>> = self.multi_get(request.keys.clone()).await?;

        // Map the internal Vec<Option<KeyValPair>> to protobuf Vec<ProtoKeyValPair>
        let pairs: Vec<ProtoKeyValPair> = request
            .keys
            .into_iter()
            .zip(results)
            .map(|(key, result)| match result {
                Some(KeyValPair(k, v)) => ProtoKeyValPair {
                    key: k,
                    val: v,
                    found: true,
                },
                None => ProtoKeyValPair {
                    key,
                    val: Vec::new(),
                    found: false,
                },
            })
            .collect();

        Ok(Response::new(MultiGetResponse { pairs }))
    }

    async fn set(&self, req: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let request: SetRequest = req.into_inner();
        let overwritten: bool = self.set(request.key, request.val).await?;
        Ok(Response::new(SetResponse { overwritten }))
    }

    async fn del(&self, req: Request<DelRequest>) -> Result<Response<DelResponse>, Status> {
        let request: DelRequest = req.into_inner();
        let absent: bool = self.del(request.key).await?;
        Ok(Response::new(DelResponse { absent }))
    }

    async fn multi_set(
        &self,
        req: Request<MultiSetRequest>,
    ) -> Result<Response<MultiSetResponse>, Status> {
        let request: MultiSetRequest = req.into_inner();

        let pairs: Vec<(Vec<u8>, Vec<u8>)> =
            request.pairs.into_iter().map(|p| (p.key, p.val)).collect();

        let overwritten_count = self.multi_set(pairs).await?;

        Ok(Response::new(MultiSetResponse { overwritten_count }))
    }

    async fn exists(
        &self,
        req: Request<ExistsRequest>,
    ) -> Result<Response<ExistsResponse>, Status> {
        let request: ExistsRequest = req.into_inner();
        let exists: bool = self.exists(request.key).await?;
        Ok(Response::new(ExistsResponse { exists }))
    }

    async fn count(&self, _: Request<CountRequest>) -> Result<Response<CountResponse>, Status> {
        let count: u64 = self.count().await?;
        Ok(Response::new(CountResponse { count }))
    }
}

pub struct MinKvsClient {
    pub client: MinKvsServiceClient<Channel>,
}

#[async_trait]
impl Kvs for MinKvsClient {
    async fn get(&self, key: Vec<u8>) -> Result<Vec<u8>, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetRequest { key });
        let response = client.get(request).await?;
        Ok(response.into_inner().val)
    }

    async fn get_keys(&self, max_keys: u32) -> Result<Vec<Vec<u8>>, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetKeysRequest {
            max_num_keys: max_keys,
        });
        let response = client.get_keys(request).await?;
        Ok(response.into_inner().keys)
    }

    async fn multi_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<KeyValPair>>, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(MultiGetRequest { keys });
        let response = client.multi_get(request).await?;
        let pairs = response.into_inner().pairs;

        // Map the protobuf ProtoKeyValPair back to the internal Option<KeyValPair>
        let results = pairs
            .into_iter()
            .map(|p| {
                if p.found {
                    Some(KeyValPair(p.key, p.val))
                } else {
                    None
                }
            })
            .collect();

        Ok(results)
    }

    async fn set(&self, key: Vec<u8>, val: Vec<u8>) -> Result<bool, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(SetRequest { key, val });
        let response = client.set(request).await?;
        Ok(response.into_inner().overwritten)
    }

    async fn del(&self, key: Vec<u8>) -> Result<bool, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(DelRequest { key });
        let response = client.del(request).await?;
        Ok(response.into_inner().absent)
    }

    async fn multi_set(&self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<u32, Status> {
        let mut client = self.client.clone();

        // Convert internal tuple pair to protobuf SetPair message
        let proto_pairs = pairs
            .into_iter()
            .map(|(key, val)| crate::minkvs::v1::SetPair { key, val })
            .collect();

        let request = tonic::Request::new(MultiSetRequest { pairs: proto_pairs });
        let response = client.multi_set(request).await?;
        Ok(response.into_inner().overwritten_count)
    }

    async fn exists(&self, key: Vec<u8>) -> Result<bool, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(ExistsRequest { key });
        let response = client.exists(request).await?;
        Ok(response.into_inner().exists)
    }

    async fn count(&self) -> Result<u64, Status> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(CountRequest {});
        let response = client.count(request).await?;
        Ok(response.into_inner().count)
    }
}
