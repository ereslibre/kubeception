# kubeception

`kubeception` is a proof of concept variant of a mix between [`bootkube`](https://github.com/kubernetes-incubator/bootkube)
and [`kubeadm`](https://github.com/kubernetes/kubernetes/tree/master/cmd/kubeadm).

It's a SUSE Hackweek 2018 project, and its purpose is to have some fun deploying a Kubernetes cluster
using Kubernetes pieces.

This project is not trying to fill any gaps in the previous pieces of software, but it's instead written from scratch to
understand how Kubernetes can launch (and maintain) Kubernetes, instead of another completely different tool to deploy
and maintain it.

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

### A note on bootstrap

Bootstrap is a safe operation that can be ran over and over. It doesn't matter the current state of the cluster, the bootstrap will
perform the following operations:

* Generate all keys and certificates needed for the cluster to start.
  * For each key and certificate, if they are present, skip.

* Set up `etcd`.
  * Render its configuration.
  * Start and enable the service.

* Write the configuration for the `kubelet`.
  * Write the `apiserver`, `replication-controller` and `scheduler` manifests in the pod manifests path of the kubelet.
    * `hostNetwork` all the way.
  * Write the `kubeconfig` for the bootstrap control plane and the "final" control plane.
  * Write the `kubelet` config pointing to the bootstrap control plane.

* Start and enable the `kubelet` service.

At this point, the `kubelet` will be talking to the `apiserver` deployed with static manifests. Of course, the `apiserver` in both
variants (bootstrap control plane and final control plane) talk to the same `etcd` backend, so in turn what we ensure by starting
a new bootstrap control plane and talk to that one while we need it; so we continue:

* Wait for the `apiserver` to be ready on its `healthz` endpoint.
  * If this is the first time, it could take a while to download the images, so we wait.
  * Even if it's not the first time, we need to ensure the apiserver is ready to continue, so we wait.

* Deploy all final control plane manifests using the bootstrap `apiserver`.
* Deploy the `kubeception` manifests using the bootstrap `apiserver`.

Now, at this point, the `kubelet` will be downloading images in order to start all the control plane pieces; or it will be starting
the containers. Note that the bootstrap `apiserver` and the final one can coexist, because they listen on different ports.

* Wait for the bootstrap `apiserver` to be ready. This is more a sanity check for when steps could be run in an independent way.
* Point the `kubelet` to the bootstrap cluster. Same thing as before.
* Wait for the `kubelet` to have registered against the `apiserver`.
* Label the node and create the taints for it (using the `bootstrap` apiserver).
* Wait for the final `apiserver` to be running in a stable manner (that is; 10 successful checks in a row).
* Point the `kubelet` to the final cluster.
* Remove the `bootstrap` control plane manifests from the static pod manifest path.

Starting our own bootstrap control plane has its advantages: we don't really care or depend on the current status of the cluster
at this moment, as long as `etcd` is healthy and can talk to us.

## Join new worker nodes

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

## Attribution

While this is a clean-room implementation (I didn't look at `bootkube` or `kubeadm` code to implement it at all), I took the
manifests and structure created by `bootkube`, and those manifests are present in this project to launch the bootstrap
control plane (`apiserver`, `controller-manager` and `scheduler`), and later on the "real" one.

## Future work

While being a recreational project I find it interesting to continue experimenting with it. Some things I'd like to continue
digging in:

* Full containerization: the `kubelet` and `etcd` are two pieces that are not containerized in this project; however it would
  be interesting to explore this. `etcd` has an [`etcd-operator` project](https://github.com/coreos/etcd-operator) that would
  make easier to properly containerize and manage `etcd` within the cluster itself.

* Safe join (implemented by `kubeadm` already). This will probably include a 2-way trust channel (so the master knows about
  the new node, and the new node can trust is talking to the right master before it tries to join).

* Upgrade the cluster: (implemented by `kubeadm` already). How a cluster upgrade would be handled with this design, and in
  the `openSUSE Kubic` context, how could we do the little orchestration needed to restart the Kubernetes workers in batches,
  so they apply their new snapshot.

* Certificates: would it make sense to use the own PKI to generate the very basic certificates to start the bootstrap server,
  and then use Kubernetes to generate the rest of certificates?

* Refactors.

## License

`kubeception` is licensed under the terms of the Apache 2.0 license.

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
