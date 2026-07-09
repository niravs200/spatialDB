use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{CertificateError, DigitallySignedStruct, DistinguishedName, Error as RustlsError, SignatureScheme};
use uuid::Uuid;
use x509_parser::prelude::*;
use x509_parser::der_parser::oid::Oid;


pub struct ReplicationCredentials {
    pub cert: CertificateDer<'static>,
    pub key: PrivateKeyDer<'static>,
}


pub fn parse_cert(bytes: &[u8]) -> Result<CertificateDer<'static>> {
    rustls_pemfile::certs(&mut &bytes[..])
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "no certificate found"))?
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        .map(|c| c.into_owned())
}

pub fn parse_key(bytes: &[u8]) -> Result<PrivateKeyDer<'static>> {
    rustls_pemfile::pkcs8_private_keys(&mut &bytes[..])
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "no key found"))?
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        .and_then(|k| PrivateKeyDer::try_from(k.secret_pkcs8_der().to_vec())
            .map_err(|e| Error::new(ErrorKind::InvalidData, e)))
}

pub fn pinned_verifier(expected_cert: CertificateDer<'static>) -> Arc<dyn ServerCertVerifier> {
    #[derive(Debug)]
    struct Verifier(CertificateDer<'static>);

    impl ServerCertVerifier for Verifier {
        fn verify_server_cert(
            &self,
            end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &ServerName<'_>,
            _ocsp: &[u8],
            _now: UnixTime,
        ) -> std::result::Result<ServerCertVerified, RustlsError> {
            if end_entity.as_ref() == self.0.as_ref() {
                Ok(ServerCertVerified::assertion())
            } else {
                Err(RustlsError::InvalidCertificate(CertificateError::UnknownIssuer))
            }
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &DigitallySignedStruct,
        ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &DigitallySignedStruct,
        ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            vec![SignatureScheme::ED25519]
        }

        fn requires_raw_public_keys(&self) -> bool {
            false
        }

        fn root_hint_subjects(&self) -> Option<&[DistinguishedName]> {
            None
        }
    }

    Arc::new(Verifier(expected_cert))
}

/// Extracts a node's unique identifier from an inbound mTLS certificate.
/// Parses the raw X.509 DER data, extracts the Subject Common Name (CN) 
/// attribute (OID 2.5.4.3), and converts its string value into a typed `Uuid`.
pub fn extract_uuid_from_cert(cert: &CertificateDer<'static>) -> Result<Uuid> {
    let (_, x509) = X509Certificate::from_der(cert.as_ref())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("X509 parse error: {e}")))?;
    
    let subject = x509.subject();
    let cn_oid = Oid::from(&[2, 5, 4, 3])
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to create OID"))?;
    
    let cn_attr = subject
        .iter_by_oid(&cn_oid)
        .next()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "CN missing from certificate subject"))?;
        
    let cn_str = cn_attr
        .as_str()
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("CN is not valid string: {e}")))?;
        
    let uuid = Uuid::parse_str(cn_str)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("CN string is not a valid UUID: {e}")))?;
    
    Ok(uuid)
}