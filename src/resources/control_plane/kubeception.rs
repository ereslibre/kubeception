pub const ALL_MANIFESTS: &'static [&'static str] = &[KUBECEPTION_SA, KUBECEPTION, KUBECEPTION_SVC];

const KUBECEPTION_SA: &'static str = r#"
apiVersion: v1
kind: ServiceAccount
metadata:
  namespace: kube-system
  name: kubeception
"#;

const KUBECEPTION: &'static str = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kubeception
  namespace: kube-system
  labels:
    tier: control-plane
    k8s-app: kubeception
spec:
  replicas: 1
  selector:
    matchLabels:
      tier: control-plane
      k8s-app: kubeception
  template:
    metadata:
      labels:
        tier: control-plane
        k8s-app: kubeception
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
                  - kubeception
              topologyKey: kubernetes.io/hostname
      containers:
      - name: kubeception
        image: {{kubeception_image}}
        command:
        - kubeception
        - serve
        - --kubeconfig=/etc/kubeconfig/kubeconfig
        livenessProbe:
          httpGet:
            path: /healthz
            port: 80
          initialDelaySeconds: 15
          timeoutSeconds: 15
        volumeMounts:
        - name: kubeconfig
          mountPath: /etc/kubeconfig
        ports:
        - name: http
          containerPort: 80
      nodeSelector:
        node-role.kubernetes.io/master: ""
      serviceAccountName: kubeception
      tolerations:
      - key: node-role.kubernetes.io/master
        operator: Exists
        effect: NoSchedule
      volumes:
      - name: kubeconfig
        configMap:
          name: kubeconfig
"#;

const KUBECEPTION_SVC: &'static str = r#"
apiVersion: v1
kind: Service
metadata:
  name: kubeception
  namespace: kube-system
  labels:
    k8s-app: kubeception
    kubernetes.io/cluster-service: "true"
    kubernetes.io/name: "kubeception"
spec:
  selector:
    k8s-app: kubeception
  type: NodePort
  ports:
  - name: kubeception
    port: 80
    protocol: TCP
    targetPort: http
    nodePort: {{kubeception_nodeport}}
"#;
