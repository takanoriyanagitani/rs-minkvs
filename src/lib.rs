pub mod minkvs {
    pub mod v1 {
        tonic::include_proto!("minkvs.v1");
    }
}

#[cfg(feature = "basic")]
pub mod basic;
