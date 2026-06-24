use std::io::{Error, ErrorKind, Result as IoResult};
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{CertificateError, DigitallySignedStruct, DistinguishedName, Error as RustlsError, SignatureScheme};

pub fn parse_cert(bytes: &[u8]) -> IoResult<CertificateDer<'static>> {
    rustls_pemfile::certs(&mut &bytes[..])
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "no certificate found"))?
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        .map(|c| c.into_owned())
}

pub fn parse_key(bytes: &[u8]) -> IoResult<PrivateKeyDer<'static>> {
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