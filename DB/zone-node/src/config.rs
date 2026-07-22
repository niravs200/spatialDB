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
    pub north: Option<NeighborConfig>,

    #[validate(nested)]
    pub south: Option<NeighborConfig>,

    #[validate(nested)]
    pub east: Option<NeighborConfig>,

    #[validate(nested)]
    pub west: Option<NeighborConfig>,
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

    if let Some(north) = &config_neighbors.north {
        let cert = parse_cert(north.quic_certificate.as_bytes())?;

        entries.insert(Direction::North, Some(NeighborInfo {
            id: north.id,
            port: north.port,
            cert,
        }));
    }

    if let Some(south) = &config_neighbors.south {
        let cert = parse_cert(south.quic_certificate.as_bytes())?;

        entries.insert(Direction::South, Some(NeighborInfo {
            id: south.id,
            port: south.port,
            cert,
        }));
    }

    if let Some(east) = &config_neighbors.east {
        let cert = parse_cert(east.quic_certificate.as_bytes())?;

        entries.insert(Direction::East, Some(NeighborInfo {
            id: east.id,
            port: east.port,
            cert,
        }));
    }

    if let Some(west) = &config_neighbors.west {
        let cert = parse_cert(west.quic_certificate.as_bytes())?;

        entries.insert(Direction::West, Some(NeighborInfo {
            id: west.id,
            port: west.port,
            cert,
        }));
    }

    Ok(RuntimeNeighbors::new(entries))
}