use openssl::rsa::Rsa;
use openssl::pkey::PKey;

use std;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use config::Config;

use openssl;
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::x509::{X509, X509Name};
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage,
                               SubjectAlternativeName};
use uuid::Uuid;

use system::{System, SystemError};

pub enum PKIError {
    OpenSSLError,
}

pub struct Key<'a> {
    name: &'a str,
    path: &'a Path,
}

pub struct CaCertificate<'a> {
    config: &'a Config,
}

pub struct Certificate<'a> {
    name: &'a str,
    path: &'a Path,
    o: &'a str,
    cn: &'a str,
    extra_ips: Vec<&'a str>,
    key: Key<'a>,
    ca: CaCertificate<'a>,
}

impl From<openssl::error::ErrorStack> for PKIError {
    fn from(_error: openssl::error::ErrorStack) -> Self {
        PKIError::OpenSSLError
    }
}

impl From<SystemError> for PKIError {
    fn from(_error: SystemError) -> Self {
        PKIError::OpenSSLError
    }
}

impl From<std::io::Error> for PKIError {
    fn from(_error: std::io::Error) -> Self {
        PKIError::OpenSSLError
    }
}

impl From<std::string::FromUtf8Error> for PKIError {
    fn from(_error: std::string::FromUtf8Error) -> Self {
        PKIError::OpenSSLError
    }
}

impl fmt::Debug for PKIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PKIError")
    }
}

impl<'a> Key<'a> {
    pub fn new<P: 'a>(name: &'a str, path: &'a P) -> Key<'a>
    where
        P: AsRef<Path>,
    {
        Key {
            name: name,
            path: path.as_ref(),
        }
    }

    fn key_path(&self) -> PathBuf {
        self.path.join(&self.name)
    }

    pub fn key(&self) -> Result<String, PKIError> {
        let mut file = File::open(self.key_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn public_key(&self) -> Result<String, PKIError> {
        Ok(String::from_utf8(
            Rsa::private_key_from_pem(self.key()?.as_bytes())?
                .public_key_to_pem()?,
        )?)
    }

    pub fn present(&self) -> Result<&Self, PKIError> {
        if self.key_path().exists() {
            return Ok(self);
        }
        let key = Rsa::generate(2048)?;
        let mut file = File::create(self.key_path())?;
        file.write_all(&key.private_key_to_pem()?)?;
        Ok(self)
    }
}

impl<'a> CaCertificate<'a> {
    pub fn new(config: &'a Config) -> CaCertificate<'a> {
        CaCertificate { config: config }
    }

    pub fn cert_path(&self) -> PathBuf {
        PathBuf::from(&self.config.secrets.path).join("ca.crt")
    }

    pub fn key(&self) -> Key {
        Key::new("ca.key", &self.config.secrets.path)
    }

    pub fn cert(&self) -> Result<X509, PKIError> {
        let mut file = File::open(self.cert_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(X509::from_pem(contents.as_bytes())?)
    }

    pub fn present(&self) -> Result<&Self, PKIError> {
        if self.cert_path().exists() {
            return Ok(self);
        }
        let mut file = File::create(self.cert_path())?;
        let cert = Certificate::create(
            &Uuid::new_v4().hyphenated().to_string(),
            &Uuid::new_v4().hyphenated().to_string(),
            &vec![],
            &Key::new("ca.key", &self.config.secrets.path),
            None,
        )?
            .to_pem()?;
        file.write_all(&cert)?;
        Ok(self)
    }
}

impl<'a> Certificate<'a> {
    pub fn new<P: 'a>(
        name: &'a str,
        path: &'a P,
        o: &'a str,
        cn: &'a str,
        extra_ips: Vec<&'a str>,
        key: Key<'a>,
        ca: CaCertificate<'a>,
    ) -> Certificate<'a>
    where
        P: AsRef<Path>,
    {
        Certificate {
            name: name,
            path: path.as_ref(),
            o: o,
            cn: cn,
            extra_ips: extra_ips,
            key: key,
            ca: ca,
        }
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn cert_path(&self) -> PathBuf {
        self.path.join(&self.name)
    }

    pub fn cert(&self) -> Result<String, PKIError> {
        let mut file = File::open(self.cert_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn create(
        o: &str,
        cn: &str,
        extra_ips: &Vec<&str>,
        key: &Key,
        ca_cert: Option<&CaCertificate>,
    ) -> Result<X509, PKIError> {
        let pkey = PKey::from_rsa(Rsa::private_key_from_pem(key.present()?.key()?.as_bytes())?)?;
        let pkey_ca = if let Some(ca_cert) = ca_cert {
            Some(PKey::from_rsa(Rsa::private_key_from_pem(
                ca_cert.key().present()?.key()?.as_bytes(),
            )?)?)
        } else {
            None
        };
        let mut name = X509Name::builder()?;
        name.append_entry_by_nid(Nid::ORGANIZATIONNAME, &o)?;
        name.append_entry_by_nid(Nid::COMMONNAME, &cn)?;
        let name = name.build();
        let mut builder = X509::builder()?;
        builder.set_version(2)?;
        builder.set_subject_name(&name)?;
        if let Some(ca_cert) = ca_cert {
            builder.set_issuer_name(&ca_cert.cert()?.issuer_name())?;
        } else {
            builder.set_issuer_name(&name)?;
        };
        builder.set_not_before(Asn1Time::days_from_now(0)?.as_ref())?;
        if ca_cert.is_some() {
            builder.set_not_after(
                Asn1Time::days_from_now(365)?.as_ref(),
            )?;
        } else {
            builder.set_not_after(
                Asn1Time::days_from_now(3650)?.as_ref(),
            )?;
        }
        builder.set_pubkey(&pkey)?;
        let mut serial = BigNum::new()?;
        serial.rand(128, MsbOption::MAYBE_ZERO, false)?;
        builder.set_serial_number(
            serial.to_asn1_integer()?.as_ref(),
        )?;
        let basic_constraints = if ca_cert.is_some() {
            BasicConstraints::new().build()?
        } else {
            BasicConstraints::new().critical().ca().build()?
        };
        builder.append_extension(basic_constraints)?;
        let key_usage = if ca_cert.is_some() {
            KeyUsage::new()
                .digital_signature()
                .key_encipherment()
                .build()?
        } else {
            KeyUsage::new()
                .digital_signature()
                .key_encipherment()
                .key_cert_sign()
                .build()?
        };
        builder.append_extension(key_usage)?;
        let ext_key_usage = ExtendedKeyUsage::new()
            .server_auth()
            .client_auth()
            .critical()
            .build()?;
        builder.append_extension(ext_key_usage)?;
        if ca_cert.is_some() {
            let mut subject_alternative_name = SubjectAlternativeName::new();
            let subject_alternative_name = subject_alternative_name.ip("127.0.0.1").dns(
                &System::hostname()?,
            );
            for extra_ip in extra_ips {
                subject_alternative_name.ip(extra_ip);
            }
            let subject_alternative_name = subject_alternative_name.build(&builder.x509v3_context(
                None,
                None,
            ))?;
            builder.append_extension(subject_alternative_name)?;
        }
        if let Some(pkey_ca) = pkey_ca {
            builder.sign(&pkey_ca, MessageDigest::sha256())?;
        } else {
            builder.sign(&pkey, MessageDigest::sha256())?;
        }
        Ok(builder.build())
    }

    pub fn present(&self) -> Result<&Self, PKIError> {
        if self.cert_path().exists() {
            return Ok(self);
        }
        let mut file = File::create(self.cert_path())?;
        let cert = Certificate::create(
            self.o,
            self.cn,
            &self.extra_ips,
            self.key.present()?,
            Some(self.ca.present()?),
        )?
            .to_pem()?;
        file.write_all(&cert)?;
        Ok(self)
    }
}
