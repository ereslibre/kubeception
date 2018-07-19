pub const KUBELET_CONFIG: &'static str = r#"KUBELET_ADDRESS="--address=127.0.0.1"
KUBELET_HOSTNAME="--hostname-override={{hostname}}"
KUBELET_ARGS="--allow-privileged=true --network-plugin=cni --cni-bin-dir=/opt/cni/bin --cni-conf-dir=/etc/kubernetes/cni/net.d --pod-manifest-path=/etc/kubernetes/manifests --volume-plugin-dir=/usr/lib --kubeconfig={{kubeconfig_path}}"
"#;
