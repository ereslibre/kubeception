use pki::*;

use base64;

use std;
use std::fs;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::{thread, time};
use std::io::Write;

use serde_json;

use resources::bootstrap::k8s::{ALL_MANIFESTS as BOOTSTRAP_MANIFESTS, KUBECONFIG};
use resources::bootstrap::kubelet::KUBELET_CONFIG as BOOTSTRAP_KUBELET_CONFIG;

use resources::control_plane::k8s::ALL_MANIFESTS as CONTROL_PLANE_MANIFESTS;
use resources::control_plane::kubelet::KUBELET_CONFIG as CONTROL_PLANE_KUBELET_CONFIG;

use resources::control_plane::kubeception::ALL_MANIFESTS as KUBECEPTION_MANIFESTS;

use config::{Config, JoinConfig};
use handlebars;
use handlebars::Handlebars;

use openssl;
use reqwest;

use system::{System, SystemError};

use systemd;
use systemd::Systemd;

use kubectl::{Kubectl, KubectlError};

pub enum Phase {
    Bootstrap,
    DeployControlPlane,
    DeployKubelet,
}

pub enum ApiserverType {
    Bootstrap,
    Cluster,
}

pub enum KubeconfigType {
    Bootstrap,
    Cluster,
}

enum WhichCertificate {
    Admin,
    ApiServer,
    EtcdClient,
}

enum WhichKey {
    Admin,
    ServiceAccount,
}

pub struct K8s<'a> {
    pub phase: Phase,
    config: &'a Config,
}

pub enum K8sError {
    UnknownError,
}

impl From<openssl::error::ErrorStack> for K8sError {
    fn from(_error: openssl::error::ErrorStack) -> Self {
        K8sError::UnknownError
    }
}

impl From<PKIError> for K8sError {
    fn from(_error: PKIError) -> K8sError {
        K8sError::UnknownError
    }
}

impl From<std::io::Error> for K8sError {
    fn from(_error: std::io::Error) -> K8sError {
        K8sError::UnknownError
    }
}

impl From<handlebars::TemplateRenderError> for K8sError {
    fn from(_error: handlebars::TemplateRenderError) -> K8sError {
        K8sError::UnknownError
    }
}

impl From<KubectlError> for K8sError {
    fn from(_error: KubectlError) -> K8sError {
        K8sError::UnknownError
    }
}

impl From<SystemError> for K8sError {
    fn from(_error: SystemError) -> K8sError {
        K8sError::UnknownError
    }
}

impl From<systemd::SystemdError> for K8sError {
    fn from(_error: systemd::SystemdError) -> K8sError {
        K8sError::UnknownError
    }
}

impl From<reqwest::Error> for K8sError {
    fn from(_error: reqwest::Error) -> K8sError {
        K8sError::UnknownError
    }
}

impl fmt::Debug for K8sError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "K8sError")
    }
}

impl<'a> K8s<'a> {
    pub fn bootstrap(config: &Config) {
        K8s::phase(Phase::Bootstrap, &config).run();
    }

    pub fn join(config: &Config, url: &String) {
        let k8s = K8s::phase(Phase::DeployKubelet, &config);
        let join_config = k8s.fetch_join_config(url).expect(
            "could not fetch join information",
        );
        k8s.write_kubeconfig(KubeconfigType::Cluster, Some(join_config))
            .expect("could not write kubeconfig information");
        k8s.write_kubelet_config(KubeconfigType::Cluster).expect(
            "could not write kubelet configuration",
        );
    }

    pub fn control_plane(config: &Config) {
        K8s::phase(Phase::DeployControlPlane, &config).run();
    }

    pub fn kubelet(config: &Config) {
        K8s::phase(Phase::DeployKubelet, &config).run();
    }

    pub fn fetch_join_config(&self, url: &String) -> Result<JoinConfig, K8sError> {
        let join_uri = reqwest::Url::parse(url).expect("could not parse the provided URL");
        let response = reqwest::get(join_uri)?.text()?;
        Ok(serde_json::from_str(&response).expect(
            "failed to process the JSON response",
        ))
    }

    pub fn apiserver_port(&self, kubeconfig_type: Option<&KubeconfigType>) -> u16 {
        match kubeconfig_type {
            Some(KubeconfigType::Bootstrap) => 6444,
            Some(KubeconfigType::Cluster) => 6443,
            None => {
                match self.phase {
                    Phase::Bootstrap |
                    Phase::DeployControlPlane => {
                        self.apiserver_port(Some(&KubeconfigType::Bootstrap))
                    }
                    Phase::DeployKubelet => self.apiserver_port(Some(&KubeconfigType::Cluster)),
                }
            }
        }
    }

    pub fn kubeconfig_path(&self, kubeconfig_type: Option<&KubeconfigType>) -> PathBuf {
        match kubeconfig_type {
            Some(KubeconfigType::Bootstrap) => {
                PathBuf::from(&self.config.secrets.path).join("kubeconfig-bootstrap")
            }
            Some(KubeconfigType::Cluster) => {
                PathBuf::from(&self.config.secrets.path).join("kubeconfig")
            }
            None => {
                match self.phase {
                    Phase::Bootstrap |
                    Phase::DeployControlPlane => {
                        self.kubeconfig_path(Some(&KubeconfigType::Bootstrap))
                    }
                    Phase::DeployKubelet => self.kubeconfig_path(Some(&KubeconfigType::Cluster)),
                }
            }
        }
    }

    fn bootstrap_manifests_path(&self) -> PathBuf {
        PathBuf::from(&self.config.bootstrap.manifests_path)
    }

    fn phase(phase: Phase, config: &'a Config) -> K8s<'a> {
        K8s {
            phase: phase,
            config: config,
        }
    }

    fn run(&self) {
        match self.phase {
            Phase::Bootstrap => {
                info!("bootstrapping");
                self.generate_certificates().expect(
                    "failed certificate generation for kubernetes",
                );
                self.write_configuration().expect(
                    "failed rendering configuration for kubernetes",
                );
                self.start_services().expect(
                    "failed starting services for kubernetes",
                );
                self.enable_services().expect(
                    "failed enabling services for kubernetes",
                );
            }
            Phase::DeployControlPlane => {
                info!("deploying control plane");
                self.deploy_control_plane().expect(
                    "failed to deploy the control plane",
                );
            }
            Phase::DeployKubelet => {
                info!("configuring local kubelet");
                self.deploy_kubelet().expect(
                    "failed to configure the local kubelet",
                );
            }
        }
    }

    fn certificate(&self, certificate: WhichCertificate) -> Certificate {
        match certificate {
            WhichCertificate::Admin => {
                Certificate::new(
                    "admin.crt",
                    &self.config.secrets.path,
                    "system:masters",
                    "admin",
                    vec![],
                    Key::new("admin.key", &self.config.secrets.path),
                    CaCertificate::new(self.config),
                )
            }
            WhichCertificate::ApiServer => {
                Certificate::new(
                    "apiserver.crt",
                    &self.config.secrets.path,
                    "kube-master",
                    "kube-apiserver",
                    vec![self.config.net.apiserver_cluster_ip.clone()],
                    Key::new("apiserver.key", &self.config.secrets.path),
                    CaCertificate::new(self.config),
                )
            }
            WhichCertificate::EtcdClient => {
                Certificate::new(
                    "etcd-client.crt",
                    &self.config.secrets.path,
                    "etcd",
                    "etcd-client",
                    vec![],
                    Key::new("etcd-client.key", &self.config.secrets.path),
                    CaCertificate::new(self.config),
                )
            }
        }
    }

    fn key(&self, key: WhichKey) -> Key {
        match key {
            WhichKey::Admin => Key::new("admin.key", &self.config.secrets.path),
            WhichKey::ServiceAccount => Key::new("service-account.key", &self.config.secrets.path),
        }
    }

    fn generate_certificates(&self) -> Result<&K8s, K8sError> {
        self.certificate(WhichCertificate::Admin).present()?;
        self.certificate(WhichCertificate::ApiServer).present()?;
        self.certificate(WhichCertificate::EtcdClient).present()?;
        let service_account_key = self.key(WhichKey::ServiceAccount).present()?.public_key()?;
        let mut file = File::create(PathBuf::from(&self.config.secrets.path).join(
            "service-account.pub",
        ))?;
        file.write_all(&service_account_key.as_bytes())?;
        Ok(self)
    }

    fn write_kubeconfig(
        &self,
        kubeconfig_type: KubeconfigType,
        join_config: Option<JoinConfig>,
    ) -> Result<&K8s, K8sError> {
        let mut file = File::create(self.kubeconfig_path(Some(&kubeconfig_type)))?;
        let config = if let Some(join_config) = join_config {
            String::from_utf8(base64::decode(&join_config.kubeconfig).expect(
                "could not decode join information",
            )).expect("invalid UTF-8")
        } else {
            self.kubeconfig_contents(kubeconfig_type)?
        };
        file.write_all(&config.as_bytes())?;
        Ok(self)
    }

    fn write_kubelet_config(&self, kubeconfig_type: KubeconfigType) -> Result<&K8s, K8sError> {
        let mut file = File::create(&self.config.kubelet.config_file)?;
        let config = self.kubelet_config_contents(kubeconfig_type)?;
        file.write_all(&config.as_bytes())?;
        Systemd::restart("kubelet.service")?;
        Ok(self)
    }

    fn kubelet_config_contents(&self, kubeconfig_type: KubeconfigType) -> Result<String, K8sError> {
        Ok(Handlebars::new().render_template(
            CONTROL_PLANE_KUBELET_CONFIG,
            &json!({
                    "hostname": System::hostname()?,
                    "kubeconfig_path": self.kubeconfig_path(Some(&kubeconfig_type)),
                }),
        )?)
    }

    fn kubeconfig_contents(&self, kubeconfig_type: KubeconfigType) -> Result<String, K8sError> {
        Ok(
            Handlebars::new().render_template(KUBECONFIG, &json!({
                "apiserver_port": self.apiserver_port(Some(&kubeconfig_type)),
                "ca_crt": base64::encode(&CaCertificate::new(self.config).cert()?.to_pem()?),
                "client_crt": base64::encode(&self.certificate(WhichCertificate::Admin).present()?.cert()?),
                "client_key": base64::encode(&self.key(WhichKey::Admin).key()?),
            }))?
        )
    }

    fn write_configuration(&self) -> Result<&K8s, K8sError> {
        match self.phase {
            Phase::Bootstrap => {
                for (name, manifest) in BOOTSTRAP_MANIFESTS {
                    let mut file = File::create(self.bootstrap_manifests_path().join(
                        format!("{}.yaml", name),
                    ))?;
                    let config = Handlebars::new().render_template(
                        manifest,
                        &json!({
                        "bootstrap_secrets_path": &self.config.secrets.path,
                        "ca_certificates_path": &self.config.secrets.path,
                    }),
                    )?;
                    file.write_all(&config.as_bytes())?;
                }

                self.write_kubeconfig(KubeconfigType::Bootstrap, None)?;
                self.write_kubeconfig(KubeconfigType::Cluster, None)?;

                {
                    let mut file = File::create(&self.config.kubelet.config_file)?;
                    file.write_all(BOOTSTRAP_KUBELET_CONFIG.as_bytes())?;
                    Systemd::restart("kubelet.service")?;
                }
            }
            Phase::DeployControlPlane |
            Phase::DeployKubelet => {}
        }
        Ok(self)
    }

    fn start_services(&self) -> Result<&K8s, K8sError> {
        Systemd::start("kubelet.service")?;
        Ok(self)
    }

    fn enable_services(&self) -> Result<&K8s, K8sError> {
        Systemd::enable("kubelet.service")?;
        Ok(self)
    }

    pub fn wait_for_apiserver(
        &self,
        apiserver_type: Option<&ApiserverType>,
    ) -> Result<&K8s, K8sError> {
        match apiserver_type {
            Some(ApiserverType::Bootstrap) => {
                info!("waiting for bootstrap apiserver");
            }
            Some(ApiserverType::Cluster) => {
                info!("waiting for cluster apiserver");
            }
            None => {
                info!("waiting for this phase default apiserver");
            }
        }
        let client = reqwest::Client::builder()
            .add_root_certificate(reqwest::Certificate::from_pem(
                CaCertificate::new(self.config)
                    .cert()?
                    .to_pem()?
                    .as_slice(),
            )?)
            .build()?;
        for _ in 1..300 {
            let mut apiserver_uri = reqwest::Url::parse("https://127.0.0.1/healthz").unwrap();
            let apiserver_port = match apiserver_type {
                Some(ApiserverType::Bootstrap) => {
                    self.apiserver_port(Some(&KubeconfigType::Bootstrap))
                }
                Some(ApiserverType::Cluster) => self.apiserver_port(Some(&KubeconfigType::Cluster)),
                None => self.apiserver_port(None),
            };
            apiserver_uri.set_port(Some(apiserver_port)).unwrap();
            let response = client.get(apiserver_uri).send();
            if response.is_ok() {
                return Ok(self);
            }
            thread::sleep(time::Duration::new(1, 0));
        }
        Err(K8sError::UnknownError)
    }

    pub fn wait_for_apiserver_stable(
        &self,
        apiserver_type: Option<&ApiserverType>,
    ) -> Result<&K8s, K8sError> {
        match apiserver_type {
            Some(ApiserverType::Bootstrap) => {
                info!("performing stability check for bootstrap apiserver");
            }
            Some(ApiserverType::Cluster) => {
                info!("performing stability check for cluster apiserver");
            }
            None => {
                info!("performing stability check for this phase default apiserver");
            }
        }
        for _ in 1..10 {
            self.wait_for_apiserver(apiserver_type)?;
            thread::sleep(time::Duration::new(1, 0));
        }
        Ok(self)
    }

    pub fn wait_for_kubelet_to_be_registered(&self) -> Result<&K8s, K8sError> {
        info!("waiting for kubelet to be registered");
        let kubectl = Kubectl::new(&self);
        for _ in 1..30 {
            if kubectl
                .run(
                    &["describe", "node", &System::hostname()?.to_owned()],
                    None,
                    Some(&KubeconfigType::Bootstrap),
                )
                .is_ok()
            {
                return Ok(self);
            }
            thread::sleep(time::Duration::new(1, 0));
        }
        Err(K8sError::UnknownError)
    }

    pub fn label_node_as_master(&self) -> Result<&K8s, K8sError> {
        info!("labeling node and setting taints");
        self.wait_for_apiserver(Some(&ApiserverType::Bootstrap))?;
        let kubectl = Kubectl::new(&self);
        kubectl.run(
            &[
                "label",
                "node",
                "--overwrite",
                &System::hostname()?.to_owned(),
                "node-role.kubernetes.io/master=",
            ],
            None,
            Some(&KubeconfigType::Bootstrap),
        )?;
        kubectl.run(
            &[
                "taint",
                "node",
                "--overwrite",
                &System::hostname()?.to_owned(),
                "node-role.kubernetes.io/master=:NoSchedule",
            ],
            None,
            Some(&KubeconfigType::Bootstrap),
        )?;
        Ok(self)
    }

    pub fn remove_static_manifests(&self) -> Result<&K8s, K8sError> {
        info!("removing static manifests");
        let manifests = fs::read_dir(self.bootstrap_manifests_path())?;
        for manifest in manifests {
            fs::remove_file(manifest?.path())?;
        }
        Ok(self)
    }

    fn deploy_manifest(&self, manifest: &str) -> Result<&K8s, K8sError> {
        let kubectl = Kubectl::new(&self);
        let processed_manifest = Handlebars::new().render_template(manifest, &json!({
            "apiserver_host": &System::hostname()?,
            "apiserver_port": self.apiserver_port(Some(&KubeconfigType::Cluster)),
            "ca_crt": base64::encode(&CaCertificate::new(self.config).cert()?.to_pem()?),
            "ca_key": base64::encode(&CaCertificate::new(self.config).key().key()?),
            "client_crt": base64::encode(&self.certificate(WhichCertificate::Admin).cert()?),
            "client_key": base64::encode(&self.certificate(WhichCertificate::Admin).key().key()?),
            "apiserver_crt": base64::encode(&self.certificate(WhichCertificate::ApiServer).cert()?),
            "apiserver_key": base64::encode(&self.certificate(WhichCertificate::ApiServer).key().key()?),
            "etcd_client_ca_crt": base64::encode(&CaCertificate::new(self.config).cert()?.to_pem()?),
            "etcd_client_crt": base64::encode(&self.certificate(WhichCertificate::EtcdClient).cert()?),
            "etcd_client_key": base64::encode(&self.certificate(WhichCertificate::EtcdClient).key().key()?),
            "service_account_key": base64::encode(&self.key(WhichKey::ServiceAccount).key()?),
            "service_account_pub": base64::encode(&self.key(WhichKey::ServiceAccount).public_key()?),
            "cluster_cidr": &self.config.net.cluster_cidr,
            "service_cluster_ip_range": &self.config.net.service_cluster_ip_range,
            "dns_cluster_ip": &self.config.net.dns_cluster_ip,
            "kubeception_image": &self.config.kubeception.image,
            "kubeception_nodeport": &self.config.kubeception.nodeport,
        }))?;
        kubectl.run(
            &["apply", "-f", "-"],
            Some(&processed_manifest),
            Some(&KubeconfigType::Bootstrap),
        )?;
        Ok(self)
    }

    pub fn deploy_control_plane(&self) -> Result<&K8s, K8sError> {
        info!("applying control plane manifests");
        self.wait_for_apiserver(Some(&ApiserverType::Bootstrap))?;
        for manifest in CONTROL_PLANE_MANIFESTS {
            self.deploy_manifest(manifest)?;
        }
        for manifest in KUBECEPTION_MANIFESTS {
            self.deploy_manifest(manifest)?;
        }
        Ok(self)
    }

    pub fn deploy_kubelet(&self) -> Result<&K8s, K8sError> {
        self.wait_for_apiserver(Some(&ApiserverType::Bootstrap))?;

        info!("pointing the kubelet to the boostrap apiserver");
        self.write_kubelet_config(KubeconfigType::Bootstrap)?;

        self.wait_for_kubelet_to_be_registered()?;
        self.label_node_as_master()?;
        self.wait_for_apiserver_stable(
            Some(&ApiserverType::Cluster),
        )?;
        self.remove_static_manifests()?;

        info!("pointing the kubelet to the cluster apiserver");
        self.write_kubelet_config(KubeconfigType::Cluster)?;

        Ok(self)
    }
}
