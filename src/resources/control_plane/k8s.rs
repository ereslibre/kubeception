pub const ALL_MANIFESTS: &'static [&'static str] = &[
    FLANNEL_CFG,
    FLANNEL_CLUSTER_ROLE_BINDING,
    FLANNEL_CLUSTER_ROLE,
    FLANNEL_SA,
    FLANNEL,
    KUBE_APISERVER_SECRET,
    KUBE_APISERVER,
    KUBECONFIG,
    KUBECONFIG_IN_CLUSTER,
    KUBE_CONTROLLER_MANAGER_DISRUPTION,
    KUBE_CONTROLLER_MANAGER_ROLE_BINDING,
    KUBE_CONTROLLER_MANAGER_SECRET,
    KUBE_CONTROLLER_MANAGER_SERVICE_ACCOUNT,
    KUBE_CONTROLLER_MANAGER,
    KUBE_DNS_DEPLOYMENT,
    KUBE_DNS_SVC,
    KUBE_PROXY_ROLE_BINDING,
    KUBE_PROXY_SA,
    KUBE_PROXY,
    KUBE_SCHEDULER_DISRUPTION,
    KUBE_SCHEDULER,
    KUBE_SYSTEM_RBAC_ROLE_BINDING,
];

const FLANNEL_CFG: &'static str = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: kube-flannel-cfg
  namespace: kube-system
  labels:
    tier: node
    k8s-app: flannel
data:
  cni-conf.json: |
    {
      "name": "cbr0",
      "cniVersion": "0.3.1",
      "plugins": [
        {
          "type": "flannel",
          "delegate": {
            "hairpinMode": true,
            "isDefaultGateway": true
          }
        },
        {
          "type": "portmap",
          "capabilities": {
            "portMappings": true
          }
        }
      ]
    }
  net-conf.json: |
    {
      "Network": "{{cluster_cidr}}",
      "Backend": {
        "Type": "vxlan",
        "Port": 4789
      }
    }
"#;

const FLANNEL_CLUSTER_ROLE_BINDING: &'static str = r#"
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: flannel
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: flannel
subjects:
- kind: ServiceAccount
  name: flannel
  namespace: kube-system
"#;

const FLANNEL_CLUSTER_ROLE: &'static str = r#"
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: flannel
rules:
  - apiGroups:
      - ""
    resources:
      - pods
    verbs:
      - get
  - apiGroups:
      - ""
    resources:
      - nodes
    verbs:
      - list
      - watch
  - apiGroups:
      - ""
    resources:
      - nodes/status
    verbs:
      - patch
"#;

const FLANNEL_SA: &'static str = r#"
apiVersion: v1
kind: ServiceAccount
metadata:
  name: flannel
  namespace: kube-system
"#;

const FLANNEL: &'static str = r#"
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: kube-flannel
  namespace: kube-system
  labels:
    tier: node
    k8s-app: flannel
spec:
  selector:
    matchLabels:
      tier: node
      k8s-app: flannel
  template:
    metadata:
      labels:
        tier: node
        k8s-app: flannel
    spec:
      serviceAccountName: flannel
      containers:
      - name: kube-flannel
        image: quay.io/coreos/flannel:v0.10.0-amd64
        command: ["/opt/bin/flanneld", "--ip-masq", "--kube-subnet-mgr", "--iface=$(POD_IP)"]
        securityContext:
          privileged: true
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: POD_IP
          valueFrom:
            fieldRef:
              fieldPath: status.podIP
        volumeMounts:
        - name: run
          mountPath: /run
        - name: cni
          mountPath: /etc/cni/net.d
        - name: flannel-cfg
          mountPath: /etc/kube-flannel/
      - name: install-cni
        image: quay.io/coreos/flannel-cni:v0.3.0
        command: ["/install-cni.sh"]
        env:
        - name: CNI_NETWORK_CONFIG
          valueFrom:
            configMapKeyRef:
              name: kube-flannel-cfg
              key: cni-conf.json
        volumeMounts:
        - name: cni
          mountPath: /host/etc/cni/net.d
        - name: host-cni-bin
          mountPath: /host/opt/cni/bin/
      hostNetwork: true
      tolerations:
      - effect: NoSchedule
        operator: Exists
      - effect: NoExecute
        operator: Exists
      volumes:
        - name: run
          hostPath:
            path: /run
        - name: cni
          hostPath:
            path: /etc/kubernetes/cni/net.d
        - name: flannel-cfg
          configMap:
            name: kube-flannel-cfg
        - name: host-cni-bin
          hostPath:
            path: /opt/cni/bin
  updateStrategy:
    rollingUpdate:
      maxUnavailable: 1
    type: RollingUpdate
"#;

const KUBE_APISERVER_SECRET: &'static str = r#"
apiVersion: v1
data:
  apiserver.crt: {{apiserver_crt}}
  apiserver.key: {{apiserver_key}}
  ca.crt: {{ca_crt}}
  etcd-client-ca.crt: {{etcd_client_ca_crt}}
  etcd-client.crt: {{etcd_client_crt}}
  etcd-client.key: {{etcd_client_key}}
  service-account.pub: {{service_account_pub}}
kind: Secret
metadata:
  name: kube-apiserver
  namespace: kube-system
type: Opaque
"#;

const KUBE_APISERVER: &'static str = r#"
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: kube-apiserver
  namespace: kube-system
  labels:
    tier: control-plane
    k8s-app: kube-apiserver
spec:
  selector:
    matchLabels:
      tier: control-plane
      k8s-app: kube-apiserver
  template:
    metadata:
      labels:
        tier: control-plane
        k8s-app: kube-apiserver
    spec:
      containers:
      - name: kube-apiserver
        image: k8s.gcr.io/hyperkube:v1.11.0
        command:
        - /hyperkube
        - apiserver
        - --enable-admission-plugins=NamespaceLifecycle,LimitRanger,ServiceAccount,DefaultTolerationSeconds,DefaultStorageClass,MutatingAdmissionWebhook,ValidatingAdmissionWebhook,ResourceQuota,NodeRestriction
        - --advertise-address=0.0.0.0
        - --allow-privileged=true
        - --anonymous-auth=false
        - --authorization-mode=Node,RBAC
        - --bind-address=0.0.0.0
        - --client-ca-file=/etc/kubernetes/secrets/ca.crt
        - --cloud-provider=
        - --etcd-cafile=/etc/kubernetes/secrets/etcd-client-ca.crt
        - --etcd-certfile=/etc/kubernetes/secrets/etcd-client.crt
        - --etcd-keyfile=/etc/kubernetes/secrets/etcd-client.key
        - --etcd-servers=https://127.0.0.1:2379
        - --insecure-port=0
        - --kubelet-client-certificate=/etc/kubernetes/secrets/apiserver.crt
        - --kubelet-client-key=/etc/kubernetes/secrets/apiserver.key
        - --secure-port=6443
        - --service-account-key-file=/etc/kubernetes/secrets/service-account.pub
        - --service-cluster-ip-range={{service_cluster_ip_range}}
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
      nodeSelector:
        node-role.kubernetes.io/master: ""
      tolerations:
      - key: node-role.kubernetes.io/master
        operator: Exists
        effect: NoSchedule
      volumes:
      - name: ssl-certs-host
        hostPath:
          path: /etc/ca-certificates
      - name: secrets
        secret:
          secretName: kube-apiserver
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
  updateStrategy:
    rollingUpdate:
      maxUnavailable: 1
    type: RollingUpdate
"#;

const KUBECONFIG: &'static str = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: kubeconfig
  namespace: kube-system
data:
  kubeconfig: |
    apiVersion: v1
    clusters:
    - name: local
      cluster:
        server: https://{{apiserver_host}}:{{apiserver_port}}
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

const KUBECONFIG_IN_CLUSTER: &'static str = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: kubeconfig-in-cluster
  namespace: kube-system
data:
  kubeconfig: |
    apiVersion: v1
    clusters:
    - name: local
      cluster:
        server: https://{{apiserver_host}}:6443
        certificate-authority: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
    users:
    - name: service-account
      user:
        # Use service account token
        tokenFile: /var/run/secrets/kubernetes.io/serviceaccount/token
    contexts:
    - context:
        cluster: local
        user: service-account
"#;

const KUBE_CONTROLLER_MANAGER_DISRUPTION: &'static str = r#"
apiVersion: policy/v1beta1
kind: PodDisruptionBudget
metadata:
  name: kube-controller-manager
  namespace: kube-system
spec:
  minAvailable: 1
  selector:
    matchLabels:
      tier: control-plane
      k8s-app: kube-controller-manager
"#;

const KUBE_CONTROLLER_MANAGER_ROLE_BINDING: &'static str = r#"
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: controller-manager
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: system:kube-controller-manager
subjects:
- kind: ServiceAccount
  name: kube-controller-manager
  namespace: kube-system
"#;

const KUBE_CONTROLLER_MANAGER_SECRET: &'static str = r#"
apiVersion: v1
data:
  ca.crt: {{ca_crt}}
  ca.key: {{ca_key}}
  service-account.key: {{service_account_key}}
kind: Secret
metadata:
  name: kube-controller-manager
  namespace: kube-system
type: Opaque
"#;

const KUBE_CONTROLLER_MANAGER_SERVICE_ACCOUNT: &'static str = r#"
apiVersion: v1
kind: ServiceAccount
metadata:
  namespace: kube-system
  name: kube-controller-manager
"#;

const KUBE_CONTROLLER_MANAGER: &'static str = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kube-controller-manager
  namespace: kube-system
  labels:
    tier: control-plane
    k8s-app: kube-controller-manager
spec:
  replicas: 2
  selector:
    matchLabels:
      tier: control-plane
      k8s-app: kube-controller-manager
  template:
    metadata:
      labels:
        tier: control-plane
        k8s-app: kube-controller-manager
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: tier
                  operator: In
                  values:
                  - control-plane
                - key: k8s-app
                  operator: In
                  values:
                  - kube-controller-manager
              topologyKey: kubernetes.io/hostname
      containers:
      - name: kube-controller-manager
        image: k8s.gcr.io/hyperkube:v1.11.0
        command:
        - ./hyperkube
        - controller-manager
        - --use-service-account-credentials
        - --allocate-node-cidrs=true
        - --cloud-provider=
        - --cluster-cidr={{cluster_cidr}}
        - --service-cluster-ip-range={{service_cluster_ip_range}}
        - --cluster-signing-cert-file=/etc/kubernetes/secrets/ca.crt
        - --cluster-signing-key-file=/etc/kubernetes/secrets/ca.key
        - --configure-cloud-routes=false
        - --leader-elect=true
        - --root-ca-file=/etc/kubernetes/secrets/ca.crt
        - --service-account-private-key-file=/etc/kubernetes/secrets/service-account.key
        livenessProbe:
          httpGet:
            path: /healthz
            port: 10252  # Note: Using default port. Update if --port option is set differently.
          initialDelaySeconds: 15
          timeoutSeconds: 15
        volumeMounts:
        - name: secrets
          mountPath: /etc/kubernetes/secrets
          readOnly: true
        - name: ssl-host
          mountPath: /etc/ssl/certs
          readOnly: true
      nodeSelector:
        node-role.kubernetes.io/master: ""
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
      serviceAccountName: kube-controller-manager
      tolerations:
      - key: node-role.kubernetes.io/master
        operator: Exists
        effect: NoSchedule
      volumes:
      - name: secrets
        secret:
          secretName: kube-controller-manager
      - name: ssl-host
        hostPath:
          path: /etc/ca-certificates
      dnsPolicy: ClusterFirstWithHostNet
"#;

const KUBE_DNS_DEPLOYMENT: &'static str = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kube-dns
  namespace: kube-system
  labels:
    k8s-app: kube-dns
    kubernetes.io/cluster-service: "true"
    addonmanager.kubernetes.io/mode: Reconcile
spec:
  # replicas: not specified here:
  # 1. In order to make Addon Manager do not reconcile this replicas parameter.
  # 2. Default is 1.
  # 3. Will be tuned in real time if DNS horizontal auto-scaling is turned on.
  strategy:
    rollingUpdate:
      maxSurge: 10%
      maxUnavailable: 0
  selector:
    matchLabels:
      k8s-app: kube-dns
  template:
    metadata:
      labels:
        k8s-app: kube-dns
    spec:
      nodeSelector:
        node-role.kubernetes.io/master: ""
      tolerations:
      - key: node-role.kubernetes.io/master
        operator: Exists
        effect: NoSchedule
      volumes:
      - name: kube-dns-config
        configMap:
          name: kube-dns
          optional: true
      containers:
      - name: kubedns
        image: k8s.gcr.io/k8s-dns-kube-dns-amd64:1.14.10
        resources:
          # TODO: Set memory limits when we've profiled the container for large
          # clusters, then set request = limit to keep this container in
          # guaranteed class. Currently, this container falls into the
          # "burstable" category so the kubelet doesn't backoff from restarting it.
          limits:
            memory: 170Mi
          requests:
            cpu: 100m
            memory: 70Mi
        livenessProbe:
          httpGet:
            path: /healthcheck/kubedns
            port: 10054
            scheme: HTTP
          initialDelaySeconds: 60
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 5
        readinessProbe:
          httpGet:
            path: /readiness
            port: 8081
            scheme: HTTP
          # we poll on pod startup for the Kubernetes master service and
          # only setup the /readiness HTTP server once that's available.
          initialDelaySeconds: 3
          timeoutSeconds: 5
        args:
        - --domain=cluster.local.
        - --dns-port=10053
        - --config-dir=/kube-dns-config
        - --v=2
        env:
        - name: PROMETHEUS_PORT
          value: "10055"
        ports:
        - containerPort: 10053
          name: dns-local
          protocol: UDP
        - containerPort: 10053
          name: dns-tcp-local
          protocol: TCP
        - containerPort: 10055
          name: metrics
          protocol: TCP
        volumeMounts:
        - name: kube-dns-config
          mountPath: /kube-dns-config
      - name: dnsmasq
        image: k8s.gcr.io/k8s-dns-dnsmasq-nanny-amd64:1.14.10
        livenessProbe:
          httpGet:
            path: /healthcheck/dnsmasq
            port: 10054
            scheme: HTTP
          initialDelaySeconds: 60
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 5
        args:
        - -v=2
        - -logtostderr
        - -configDir=/etc/k8s/dns/dnsmasq-nanny
        - -restartDnsmasq=true
        - --
        - -k
        - --cache-size=1000
        - --no-negcache
        - --log-facility=-
        - --server=/cluster.local/127.0.0.1#10053
        - --server=/in-addr.arpa/127.0.0.1#10053
        - --server=/ip6.arpa/127.0.0.1#10053
        ports:
        - containerPort: 53
          name: dns
          protocol: UDP
        - containerPort: 53
          name: dns-tcp
          protocol: TCP
        # see: https://github.com/kubernetes/kubernetes/issues/29055 for details
        resources:
          requests:
            cpu: 150m
            memory: 20Mi
        volumeMounts:
        - name: kube-dns-config
          mountPath: /etc/k8s/dns/dnsmasq-nanny
      - name: sidecar
        image: k8s.gcr.io/k8s-dns-sidecar-amd64:1.14.10
        livenessProbe:
          httpGet:
            path: /metrics
            port: 10054
            scheme: HTTP
          initialDelaySeconds: 60
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 5
        args:
        - --v=2
        - --logtostderr
        - --probe=kubedns,127.0.0.1:10053,kubernetes.default.svc.cluster.local,5,SRV
        - --probe=dnsmasq,127.0.0.1:53,kubernetes.default.svc.cluster.local,5,SRV
        ports:
        - containerPort: 10054
          name: metrics
          protocol: TCP
        resources:
          requests:
            memory: 20Mi
            cpu: 10m
      dnsPolicy: Default  # Don't use cluster DNS.
"#;

const KUBE_DNS_SVC: &'static str = r#"
apiVersion: v1
kind: Service
metadata:
  name: kube-dns
  namespace: kube-system
  labels:
    k8s-app: kube-dns
    kubernetes.io/cluster-service: "true"
    kubernetes.io/name: "KubeDNS"
spec:
  selector:
    k8s-app: kube-dns
  clusterIP: {{dns_cluster_ip}}
  ports:
  - name: dns
    port: 53
    protocol: UDP
  - name: dns-tcp
    port: 53
    protocol: TCP
"#;

const KUBE_PROXY_ROLE_BINDING: &'static str = r#"
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: kube-proxy
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: system:node-proxier # Automatically created system role.
subjects:
- kind: ServiceAccount
  name: kube-proxy
  namespace: kube-system
"#;

const KUBE_PROXY_SA: &'static str = r#"
apiVersion: v1
kind: ServiceAccount
metadata:
  namespace: kube-system
  name: kube-proxy
"#;

const KUBE_PROXY: &'static str = r#"
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: kube-proxy
  namespace: kube-system
  labels:
    tier: node
    k8s-app: kube-proxy
spec:
  selector:
    matchLabels:
      tier: node
      k8s-app: kube-proxy
  template:
    metadata:
      labels:
        tier: node
        k8s-app: kube-proxy
    spec:
      containers:
      - name: kube-proxy
        image: k8s.gcr.io/hyperkube:v1.11.0
        command:
        - ./hyperkube
        - proxy
        - --cluster-cidr={{cluster_cidr}}
        - --hostname-override=$(NODE_NAME)
        - --kubeconfig=/etc/kubernetes/kubeconfig
        - --proxy-mode=iptables
        env:
          - name: NODE_NAME
            valueFrom:
              fieldRef:
                fieldPath: spec.nodeName
        securityContext:
          privileged: true
        volumeMounts:
        - mountPath: /lib/modules
          name: lib-modules
          readOnly: true
        - mountPath: /etc/ssl/certs
          name: ssl-certs-host
          readOnly: true
        - name: kubeconfig
          mountPath: /etc/kubernetes
          readOnly: true
      hostNetwork: true
      serviceAccountName: kube-proxy
      tolerations:
      - effect: NoSchedule
        operator: Exists
      - effect: NoExecute
        operator: Exists
      volumes:
      - name: lib-modules
        hostPath:
          path: /lib/modules
      - name: ssl-certs-host
        hostPath:
          path: /etc/ca-certificates
      - name: kubeconfig
        configMap:
          name: kubeconfig-in-cluster
  updateStrategy:
    rollingUpdate:
      maxUnavailable: 1
    type: RollingUpdate
"#;

const KUBE_SCHEDULER_DISRUPTION: &'static str = r#"
apiVersion: policy/v1beta1
kind: PodDisruptionBudget
metadata:
  name: kube-scheduler
  namespace: kube-system
spec:
  minAvailable: 1
  selector:
    matchLabels:
      tier: control-plane
      k8s-app: kube-scheduler
"#;

const KUBE_SCHEDULER: &'static str = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kube-scheduler
  namespace: kube-system
  labels:
    tier: control-plane
    k8s-app: kube-scheduler
spec:
  replicas: 2
  selector:
    matchLabels:
      tier: control-plane
      k8s-app: kube-scheduler
  template:
    metadata:
      labels:
        tier: control-plane
        k8s-app: kube-scheduler
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: tier
                  operator: In
                  values:
                  - control-plane
                - key: k8s-app
                  operator: In
                  values:
                  - kube-scheduler
              topologyKey: kubernetes.io/hostname
      containers:
      - name: kube-scheduler
        image: k8s.gcr.io/hyperkube:v1.11.0
        command:
        - ./hyperkube
        - scheduler
        - --leader-elect=true
        livenessProbe:
          httpGet:
            path: /healthz
            port: 10251  # Note: Using default port. Update if --port option is set differently.
          initialDelaySeconds: 15
          timeoutSeconds: 15
      nodeSelector:
        node-role.kubernetes.io/master: ""
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
      tolerations:
      - key: node-role.kubernetes.io/master
        operator: Exists
        effect: NoSchedule
"#;

const KUBE_SYSTEM_RBAC_ROLE_BINDING: &'static str = r#"
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: system:default-sa
subjects:
  - kind: ServiceAccount
    name: default
    namespace: kube-system
roleRef:
  kind: ClusterRole
  name: cluster-admin
  apiGroup: rbac.authorization.k8s.io
"#;
