use pki::*;
use std::fs;

use std;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use config::Config;
use handlebars;
use handlebars::Handlebars;

use systemd;
use systemd::Systemd;
use resources::bootstrap::etcd::ETCD_BOOTSTRAP_CONFIG;

pub enum Phase {
    Bootstrap,
}

pub struct Etcd<'a> {
    phase: Phase,
    config: &'a Config,
}

pub enum EtcdError {
    UnknownError,
}

impl From<PKIError> for EtcdError {
    fn from(_error: PKIError) -> EtcdError {
        EtcdError::UnknownError
    }
}

impl From<std::io::Error> for EtcdError {
    fn from(_error: std::io::Error) -> EtcdError {
        EtcdError::UnknownError
    }
}

impl From<handlebars::TemplateRenderError> for EtcdError {
    fn from(_error: handlebars::TemplateRenderError) -> EtcdError {
        EtcdError::UnknownError
    }
}

impl From<systemd::SystemdError> for EtcdError {
    fn from(_error: systemd::SystemdError) -> EtcdError {
        EtcdError::UnknownError
    }
}

impl fmt::Debug for EtcdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EtcdError")
    }
}

impl<'a> Etcd<'a> {
    pub fn bootstrap(config: &Config) {
        Etcd::phase(Phase::Bootstrap, &config).run();
    }

    fn config_path(&self) -> PathBuf {
        PathBuf::from(&self.config.etcd.config_path)
    }

    fn phase(phase: Phase, config: &'a Config) -> Etcd<'a> {
        Etcd {
            phase: phase,
            config: config,
        }
    }

    fn run(&self) {
        match self.phase {
            Phase::Bootstrap => {
                info!("bootstrapping");
                if !Path::new(&self.config.etcd.config_path).is_dir() {
                    fs::create_dir(&self.config.etcd.config_path).expect(
                        "failed to create etcd config directory",
                    );
                }
                self.generate_certificates().expect(
                    "failed certificate generation for etcd",
                );
                self.write_configuration().expect(
                    "failed rendering configuration for etcd",
                );
                self.start_services().expect(
                    "failed starting services for etcd",
                );
                self.enable_services().expect(
                    "failed enabling services for etcd",
                );
            }
        }
    }

    fn generate_certificates(&self) -> Result<&Etcd, EtcdError> {
        Certificate::new(
            "peer.crt",
            &self.config.etcd.config_path,
            "etcd",
            "etcd-peer",
            vec![],
            Key::new("peer.key", &self.config.etcd.config_path),
            CaCertificate::new(self.config),
        ).present()?;
        Certificate::new(
            "server.crt",
            &self.config.etcd.config_path,
            "etcd",
            "etcd-server",
            vec![],
            Key::new("server.key", &self.config.etcd.config_path),
            CaCertificate::new(self.config),
        ).present()?;
        Ok(self)
    }

    fn write_configuration(&self) -> Result<&Etcd, EtcdError> {
        match self.phase {
            Phase::Bootstrap => {
                let mut reg = Handlebars::new();
                let mut file = File::create(&self.config.etcd.config_file)?;
                let config = reg.render_template(
                    ETCD_BOOTSTRAP_CONFIG,
                    &json!({
                    "etcd_ca_file_path": CaCertificate::new(&self.config).cert_path(),
                    "etcd_server_cert_file_path": self.config_path().join("server.crt"),
                    "etcd_server_key_file_path": self.config_path().join("server.key"),
                    "etcd_peer_cert_file_path": self.config_path().join("peer.crt"),
                    "etcd_peer_key_file_path": self.config_path().join("peer.key"),
                }),
                )?;
                file.write_all(&config.as_bytes())?;
            }
        }
        Ok(self)
    }

    fn start_services(&self) -> Result<&Etcd, EtcdError> {
        Systemd::start("etcd.service")?;
        Ok(self)
    }

    fn enable_services(&self) -> Result<&Etcd, EtcdError> {
        Systemd::enable("etcd.service")?;
        Ok(self)
    }
}
