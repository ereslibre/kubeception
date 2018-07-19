pub const ALL_MANIFESTS: &'static [(&'static str, &'static str)] =
    &[
        ("apiserver", KUBE_APISERVER_MANIFEST),
        ("controller_manager", KUBE_CONTROLLER_MANAGER_MANIFEST),
        ("scheduler", KUBE_SCHEDULER_MANIFEST),
    ];

const KUBE_APISERVER_MANIFEST: &'static str = r#"apiVersion: v1
kind: Pod
metadata:
  name: bootstrap-kube-apiserver
  namespace: kube-system
spec:
  containers:
  - name: kube-apiserver
    image: k8s.gcr.io/hyperkube:v1.11.0
    command:
    - /hyperkube
    - apiserver
    - --advertise-address=0.0.0.0
    - --allow-privileged=true
    - --authorization-mode=Node,RBAC
    - --bind-address=0.0.0.0
    - --client-ca-file=/etc/kubernetes/secrets/ca.crt
    - --enable-admission-plugins=NamespaceLifecycle,LimitRanger,ServiceAccount,DefaultTolerationSeconds,DefaultStorageClass,MutatingAdmissionWebhook,ValidatingAdmissionWebhook,ResourceQuota
    - --enable-bootstrap-token-auth=true
    - --etcd-cafile=/etc/kubernetes/secrets/ca.crt
    - --etcd-certfile=/etc/kubernetes/secrets/etcd-client.crt
    - --etcd-keyfile=/etc/kubernetes/secrets/etcd-client.key
    - --etcd-servers=https://127.0.0.1:2379
    - --kubelet-client-certificate=/etc/kubernetes/secrets/apiserver.crt
    - --kubelet-client-key=/etc/kubernetes/secrets/apiserver.key
    - --secure-port=6444
    - --service-account-key-file=/etc/kubernetes/secrets/service-account.pub
    - --service-cluster-ip-range=10.3.0.0/24
    - --cloud-provider=
    - --storage-backend=etcd3
    - --tls-cert-file=/etc/kubernetes/secrets/apiserver.crt
    - --tls-private-key-file=/etc/kubernetes/secrets/apiserver.key
    env:
    - name: POD_IP
      valueFrom:
        fieldRef:
          fieldPath: status.podIP
    volumeMounts:
    - mountPath: /etc/ssl/certs
      name: ssl-certs-host
      readOnly: true
    - mountPath: /etc/kubernetes/secrets
      name: secrets
      readOnly: true
  hostNetwork: true
  volumes:
  - name: secrets
    hostPath:
      path: {{bootstrap_secrets_path}}
  - name: ssl-certs-host
    hostPath:
      path: {{ca_certificates_path}}
"#;

const KUBE_CONTROLLER_MANAGER_MANIFEST: &'static str = r#"apiVersion: v1
kind: Pod
metadata:
  name: bootstrap-kube-controller-manager
  namespace: kube-system
spec:
  containers:
  - name: kube-controller-manager
    image: k8s.gcr.io/hyperkube:v1.11.0
    command:
    - ./hyperkube
    - controller-manager
    - --allocate-node-cidrs=true
    - --cluster-cidr=10.2.0.0/16
    - --service-cluster-ip-range=10.3.0.0/24
    - --cloud-provider=
    - --cluster-signing-cert-file=/etc/kubernetes/secrets/ca.crt
    - --cluster-signing-key-file=/etc/kubernetes/secrets/ca.key
    - --configure-cloud-routes=false
    - --kubeconfig=/etc/kubernetes/secrets/kubeconfig-bootstrap
    - --leader-elect=true
    - --root-ca-file=/etc/kubernetes/secrets/ca.crt
    - --service-account-private-key-file=/etc/kubernetes/secrets/service-account.key
    volumeMounts:
    - name: secrets
      mountPath: /etc/kubernetes/secrets
      readOnly: true
    - name: ssl-host
      mountPath: /etc/ssl/certs
      readOnly: true
  hostNetwork: true
  volumes:
  - name: secrets
    hostPath:
      path: {{bootstrap_secrets_path}}
  - name: ssl-host
    hostPath:
      path: {{ca_certificates_path}}
"#;

const KUBE_SCHEDULER_MANIFEST: &'static str = r#"apiVersion: v1
kind: Pod
metadata:
  name: bootstrap-kube-scheduler
  namespace: kube-system
spec:
  containers:
  - name: kube-scheduler
    image: k8s.gcr.io/hyperkube:v1.11.0
    command:
    - ./hyperkube
    - scheduler
    - --kubeconfig=/etc/kubernetes/secrets/kubeconfig-bootstrap
    - --leader-elect=true
    volumeMounts:
    - name: secrets
      mountPath: /etc/kubernetes/secrets
      readOnly: true
  hostNetwork: true
  volumes:
  - name: secrets
    hostPath:
      path: {{bootstrap_secrets_path}}
"#;

pub const KUBECONFIG: &'static str = r#"apiVersion: v1
kind: Config
clusters:
- name: local
  cluster:
    server: https://127.0.0.1:{{apiserver_port}}
    certificate-authority-data: {{ca_crt}}
users:
- name: admin
  user:
    client-certificate-data: {{client_crt}}
    client-key-data: {{client_key}}
contexts:
- context:
    cluster: local
    user: admin
"#;
