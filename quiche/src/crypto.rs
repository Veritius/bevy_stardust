use boring::x509::X509;

#[derive(Clone)]
pub struct Certificate {
    inner: X509,
}