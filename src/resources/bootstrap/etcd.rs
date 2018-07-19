pub const ETCD_BOOTSTRAP_CONFIG: &'static str = r#"ETCD_NAME=default
ETCD_DATA_DIR="/var/lib/etcd/default.etcd"
ETCD_LISTEN_CLIENT_URLS="https://0.0.0.0:2379"
ETCD_ADVERTISE_CLIENT_URLS="https://0.0.0.0:2379"
ETCD_CLIENT_CERT_AUTH="true"

ETCD_CA_FILE={{etcd_ca_file_path}}
ETCD_CERT_FILE={{etcd_server_cert_file_path}}
ETCD_KEY_FILE={{etcd_server_key_file_path}}
ETCD_TRUSTED_CA_FILE={{etcd_ca_file_path}}

ETCD_PEER_CA_FILE={{etcd_ca_file_path}}
ETCD_PEER_CERT_FILE={{etcd_peer_cert_file_path}}
ETCD_PEER_KEY_FILE={{etcd_peer_key_file_path}}
ETCD_PEER_TRUSTED_CA_FILE={{etcd_ca_file_path}}
ETCD_PEER_CLIENT_CERT_AUTH="true"
"#;
