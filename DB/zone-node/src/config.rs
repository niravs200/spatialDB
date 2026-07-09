use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[validate(range(min = 1, max = 65535))]
    pub client_port: u16,

    #[validate(range(min = 1, max = 65535))]
    pub realtime_port: u16,

    #[validate(range(min = 1, max = 65535))]
    pub control_port: u16,

    #[validate(range(min = 1, max = 65535))]
    pub replication_port: u16,

    #[validate(nested)]
    pub neighbors: Neighbors,

    pub quic_certificate: String,
    pub quic_private_key: String,
}


#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Neighbors {
    #[validate(nested)]
    pub north: NeighborConfig,

    #[validate(nested)]
    pub south: NeighborConfig,

    #[validate(nested)]
    pub east: NeighborConfig,

    #[validate(nested)]
    pub west: NeighborConfig,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct NeighborConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    pub certificate: String,
}