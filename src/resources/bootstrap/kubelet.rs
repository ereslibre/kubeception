pub const KUBELET_CONFIG: &'static str = r#"KUBELET_ADDRESS="--address=127.0.0.1"
KUBELET_ARGS="--pod-manifest-path=/etc/kubernetes/manifests --volume-plugin-dir=/usr/lib"
"#;
