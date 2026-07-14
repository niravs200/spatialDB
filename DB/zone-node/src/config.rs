use std::collections::HashMap;
use serde::Deserialize;
use validator::{Validate, ValidationError};
use uuid::Uuid;
use std::io::{Error, ErrorKind, Result};

use crate::certificate::parse_cert;
use crate::metadata::{Direction, NeighborInfo, Neighbors as RuntimeNeighbors};

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub id: Uuid,

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
    pub id: Uuid,

    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    #[validate(custom(function = "validate_is_pem_wrapper"))]
    pub quic_certificate: String,
}

fn validate_is_pem(cert_str: &str) -> Result<()> {
    if cert_str.contains("-----BEGIN CERTIFICATE-----") && cert_str.contains("-----END CERTIFICATE-----") {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidData, "invalid PEM formatting"))
    }
}

fn validate_is_pem_wrapper(cert_str: &str) -> std::result::Result<(), ValidationError> {
    validate_is_pem(cert_str).map_err(|_| ValidationError::new("invalid_pem_format"))
}

pub fn extract_runtime_neighbors(config_neighbors: &Neighbors) -> Result<RuntimeNeighbors> {
    let mut entries = HashMap::new();

    let north_cert = parse_cert(config_neighbors.north.quic_certificate.as_bytes())?;
    entries.insert(Direction::North, NeighborInfo {
        id: config_neighbors.north.id,
        port: config_neighbors.north.port,
        cert: north_cert,
    });

    let south_cert = parse_cert(config_neighbors.south.quic_certificate.as_bytes())?;
    entries.insert(Direction::South, NeighborInfo {
        id: config_neighbors.south.id,
        port: config_neighbors.south.port,
        cert: south_cert,
    });

    let east_cert = parse_cert(config_neighbors.east.quic_certificate.as_bytes())?;
    entries.insert(Direction::East, NeighborInfo {
        id: config_neighbors.east.id,
        port: config_neighbors.east.port,
        cert: east_cert,
    });

    let west_cert = parse_cert(config_neighbors.west.quic_certificate.as_bytes())?;
    entries.insert(Direction::West, NeighborInfo {
        id: config_neighbors.west.id,
        port: config_neighbors.west.port,
        cert: west_cert,
    });

    Ok(RuntimeNeighbors::new(entries))
}