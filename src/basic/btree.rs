use std::sync::RwLock;

use std::collections::BTreeMap;

use tonic::Status;
use tonic::async_trait;

use crate::basic::core::{KeyValPair, Kvs};

#[derive(Default)]
pub struct KvsBtree {
    rwmap: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
}

#[async_trait]
impl Kvs for KvsBtree {
    async fn get(&self, key: Vec<u8>) -> Result<Vec<u8>, Status> {
        // Acquire read lock
        let map = self
            .rwmap
            .read()
            .map_err(|_| Status::internal("unable to read lock"))?;

        map.get(&key)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("Key {:?} not found", key)))
    }

    async fn get_keys(&self, max_keys: u32) -> Result<Vec<Vec<u8>>, Status> {
        let map = self
            .rwmap
            .read()
            .map_err(|_| Status::internal("unable to read lock"))?;

        // BTreeMap keys are already sorted
        let keys = map.keys().take(max_keys as usize).cloned().collect();

        Ok(keys)
    }

    async fn multi_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<KeyValPair>>, Status> {
        let map = self
            .rwmap
            .read()
            .map_err(|_| Status::internal("unable to read lock"))?;

        let mapd = keys.into_iter().map(|key| {
            let oval: Option<&Vec<u8>> = map.get(&key);
            oval.map(|val| KeyValPair(key, val.clone()))
        });

        Ok(mapd.collect())
    }

    async fn set(&self, key: Vec<u8>, val: Vec<u8>) -> Result<bool, Status> {
        // Acquire write lock
        let mut map = self
            .rwmap
            .write()
            .map_err(|_| Status::internal("unable to write lock"))?;

        // BTreeMap::insert returns Some(old_value) if the key already existed
        let overwritten = map.insert(key, val).is_some();
        Ok(overwritten)
    }

    async fn del(&self, key: Vec<u8>) -> Result<bool, Status> {
        // Acquire write lock
        let mut map = self
            .rwmap
            .write()
            .map_err(|_| Status::internal("unable to write lock"))?;

        // BTreeMap::remove returns Some(value) if the key was present
        // The trait requires returning true if it was ABSENT
        let absent = map.remove(&key).is_none();
        Ok(absent)
    }

    async fn multi_set(&self, pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<u32, Status> {
        let mut map = self
            .rwmap
            .write()
            .map_err(|_| Status::internal("unable to write lock"))?;

        let mut overwritten_count = 0;
        for (key, val) in pairs {
            if map.insert(key, val).is_some() {
                overwritten_count += 1;
            }
        }
        Ok(overwritten_count)
    }

    async fn exists(&self, key: Vec<u8>) -> Result<bool, Status> {
        let map = self
            .rwmap
            .read()
            .map_err(|_| Status::internal("unable to read lock"))?;

        Ok(map.contains_key(&key))
    }

    async fn count(&self) -> Result<u64, Status> {
        let map = self
            .rwmap
            .read()
            .map_err(|_| Status::internal("unable to read lock"))?;

        Ok(map.len() as u64)
    }
}
