use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use toml;

#[derive(Deserialize)]
pub struct Config {
    pub bootstrap: Bootstrap,
    pub certificates: Certificates,
    pub secrets: Secrets,
    pub kubelet: Kubelet,
    pub etcd: Etcd,
    pub net: Net,
    pub kubeception: Kubeception,
}

#[derive(Deserialize)]
pub struct Bootstrap {
    pub manifests_path: String,
}

#[derive(Deserialize)]
pub struct Certificates {
    pub ca_path: String,
}

#[derive(Deserialize)]
pub struct Secrets {
    pub path: String,
}

#[derive(Deserialize)]
pub struct Kubelet {
    pub config_file: String,
}

#[derive(Deserialize)]
pub struct Etcd {
    pub config_file: String,
    pub config_path: String,
}

#[derive(Deserialize)]
pub struct Net {
    pub cluster_cidr: String,
    pub service_cluster_ip_range: String,
    pub apiserver_cluster_ip: String,
    pub dns_cluster_ip: String,
}

#[derive(Deserialize)]
pub struct Kubeception {
    pub image: String,
    pub nodeport: String,
}

#[derive(Serialize, Deserialize)]
pub struct JoinConfig {
    pub kubeconfig: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Config
    {
        let mut file = File::open(path).expect("configuration file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(
            "could not read configuration file",
        );
        toml::from_str(&contents).expect("could not deserialize configuration file")
    }
}
