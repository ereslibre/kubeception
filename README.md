# kubeception

`kubeception` is a proof of concept variant of a mix between [`bootkube`](https://github.com/kubernetes-incubator/bootkube)
and [`kubeadm`](https://github.com/kubernetes/kubernetes/tree/master/cmd/kubeadm).

It's a SUSE Hackweek 2018 project, and its purpose is to have some fun deploying a Kubernetes cluster
using Kubernetes pieces.

## Launch kubeception

`kubeception` assumes some things on the host system:

* It has `etcd` installed.
* It has `kubelet` installed.
* It has `kubectl` installed.
* It has a container runtime available.

`kubeception` supports different distro-targeted configurations. One is provided for [`openSUSE Kubic`](https://kubic.opensuse.org/).

That being said, a simple way to launch `kubeception` in an `openSUSE Kubic` environment would be:

```
docker run -v /usr/bin/kubectl:/usr/bin/kubectl \
           -v /etc/kubernetes/kubelet:/etc/kubernetes/kubelet \
           -v /etc/sysconfig/etcd:/etc/sysconfig/etcd \
           -v /etc/etcd:/etc/etcd \
           -v /etc/kubernetes/manifests:/etc/kubernetes/manifests \
           -v /etc/kubernetes/bootstrap-secrets:/etc/kubernetes/bootstrap-secrets \
           -v /var/run/dbus:/var/run/dbus \
           -e "RUST_LOG=info" --net=host \
           -it ereslibre/kubeception kubeception bootstrap --config config/kubic.toml
```

In less than a minute, our node has become a Kubernetes master:

```
 INFO 2018-07-18T01:23:46Z: kubeception::etcd: bootstrapping
 INFO 2018-07-18T01:23:46Z: kubeception::k8s: bootstrapping
 INFO 2018-07-18T01:23:47Z: kubeception::k8s: deploying control plane
 INFO 2018-07-18T01:23:47Z: kubeception::k8s: applying control plane manifests
 INFO 2018-07-18T01:23:47Z: kubeception::k8s: waiting for bootstrap apiserver
 INFO 2018-07-18T01:24:04Z: kubeception::k8s: configuring local kubelet
 INFO 2018-07-18T01:24:04Z: kubeception::k8s: waiting for bootstrap apiserver
 INFO 2018-07-18T01:24:04Z: kubeception::k8s: pointing the kubelet to the boostrap apiserver
 INFO 2018-07-18T01:24:04Z: kubeception::k8s: waiting for kubelet to be registered
 INFO 2018-07-18T01:24:05Z: kubeception::k8s: labeling node and setting taints
 INFO 2018-07-18T01:24:05Z: kubeception::k8s: waiting for bootstrap apiserver
 INFO 2018-07-18T01:24:05Z: kubeception::k8s: performing stability check for cluster apiserver
 INFO 2018-07-18T01:24:05Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:25Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:26Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:27Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:28Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:29Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:30Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:31Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:32Z: kubeception::k8s: waiting for cluster apiserver
 INFO 2018-07-18T01:24:33Z: kubeception::k8s: removing static manifests
 INFO 2018-07-18T01:24:33Z: kubeception::k8s: pointing the kubelet to the cluster apiserver
```

You can check that everything is running as expected, and that the node is now `Ready`:

```
linux-e9u2:~ # kubectl --kubeconfig=/etc/kubernetes/bootstrap-secrets/kubeconfig get nodes -o wide
NAME         STATUS    ROLES     AGE       VERSION   EXTERNAL-IP   OS-IMAGE                    KERNEL-VERSION     CONTAINER-RUNTIME
linux-e9u2   Ready     master    41m       v1.10.4   <none>        openSUSE Tumbleweed Kubic   4.17.4-1-default   docker://17.9.1
```

All pods should be in `Running` state as expected:

```
linux-e9u2:~ # kubectl --kubeconfig=/etc/kubernetes/bootstrap-secrets/kubeconfig get pods --all-namespaces
NAMESPACE     NAME                                       READY     STATUS    RESTARTS   AGE
kube-system   kube-apiserver-qb75b                       1/1       Running   1          42m
kube-system   kube-controller-manager-86c48ffdbd-qh8jh   1/1       Running   1          42m
kube-system   kube-controller-manager-86c48ffdbd-qvwrj   1/1       Running   1          42m
kube-system   kube-dns-5d84c799b5-2dn2q                  3/3       Running   4          42m
kube-system   kube-flannel-pf5qr                         2/2       Running   1          4m
kube-system   kube-flannel-xwdr5                         2/2       Running   4          42m
kube-system   kube-proxy-2wfn2                           1/1       Running   1          42m
kube-system   kube-proxy-bk4nh                           1/1       Running   0          4m
kube-system   kube-scheduler-54d5cb78b8-nd4ds            1/1       Running   1          42m
kube-system   kube-scheduler-54d5cb78b8-pxbq6            1/1       Running   1          42m
kube-system   kubeception-6f7d75db5d-rnhks               1/1       Running   1          42m
```

We have successfully `kubeception`'ed our `openSUSE Kubic` node!

## Join new (worker) nodes

You can join new worker nodes to your existing Kubernetes installation. `kubeception` did create a deployment
of itself inside the cluster with an exposed node port. This means that you can reach to any machine at the
`30000` port, and join the cluster.

BIG NOTE: NO SECURITY AT ALL. HTTP TRANSPORT, NO TOKENS, NOTHING (yet).

We also assume some things about your worker nodes:

* It has `kubelet` installed.
* It has a container runtime available.

Now, let's join the cluster. From the machine to be joined, run:

```
docker run -v /etc/kubernetes/kubelet:/etc/kubernetes/kubelet \
           -v /etc/kubernetes/bootstrap-secrets:/etc/kubernetes/bootstrap-secrets \
           -v /var/run/dbus:/var/run/dbus \
           -e "RUST_LOG=info" --net=host \
           -it ereslibre/kubeception kubeception join --url http://linux-e9u2:30000/join --config config/kubic.toml
```

Now the node should have joined, it's an instant operation. It will become `Ready` when CNI is finally deployed on it (some images
need to be downloaded):

```
linux-e9u2:~ # kubectl --kubeconfig=/etc/kubernetes/bootstrap-secrets/kubeconfig get nodes -o wide
NAME         STATUS    ROLES     AGE       VERSION   EXTERNAL-IP   OS-IMAGE                    KERNEL-VERSION     CONTAINER-RUNTIME
linux-1n7n   Ready     <none>    3m        v1.10.4   <none>        openSUSE Tumbleweed Kubic   4.17.4-1-default   docker://17.9.1
linux-e9u2   Ready     master    41m       v1.10.4   <none>        openSUSE Tumbleweed Kubic   4.17.4-1-default   docker://17.9.1
```
