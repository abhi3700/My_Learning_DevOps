# Docker

Learn everything about Docker.

## About

- Build, run & ship applications.
- It helps in containerization
- Since Docker Engine only runs on Linux, developers who use Windows and macOS for software development cannot run the engine until they spin up a virtual machine (VM) that runs linux.

## Legend

```bash
❯: mac
$: linux VM
```

## Installation

### macOS

#### 1. Directly on Host machine

**docker**

- `❯ brew install docker`
  > if exists, update using `$ brew upgrade docker`

---

**docker-machine**

- `❯ brew install docker-machine`
  > if exists, update using `$ brew upgrade docker-machine`

> `brew` on macOS is like `apt` on Ubuntu.

---

> On macOS the docker binary is only a client and you cannot use it to run the docker daemon, because Docker daemon uses Linux-specific kernel features, therefore you can’t run Docker natively in OS X. So you have to install docker-machine in order to create VM and attach to it.

`docker-machine` relies on VirtualBox being installed and will fail if this isn't the case.

---

**Install VirtualBox**

```console
❯ brew install virtualbox
==> Downloading https://download.virtualbox.org/virtualbox/7.0.2/VirtualBox-7.0.
Already downloaded: /Users/abhi3700/Library/Caches/Homebrew/downloads/208eb45ad7f80d3564e4de7d8bd64eefbf72aef4ea004f68957f55341724bb0e--VirtualBox-7.0.2-154219-OSX.dmg
Error: Cask virtualbox depends on hardware architecture being one of [{:type=>:intel, :bits=>64}], but you are running {:type=>:arm, :bits=>64}.
```

And then shift to intel, x64 architecture by setting up Rosetta. So, just use the command `$ intel` in the terminal to switch to intel arch.

[Reference](https://docs.docker.com/desktop/install/mac-install/)
![](../img/mac_m1_docker.png)

```console
❯ intel
❯ arch
i386
```

Retry:

```console
❯ brew install virtualbox
==> Downloading https://download.virtualbox.org/virtualbox/7.0.2/VirtualBox-7.0.
Already downloaded: /Users/abhi3700/Library/Caches/Homebrew/downloads/208eb45ad7f80d3564e4de7d8bd64eefbf72aef4ea004f68957f55341724bb0e--VirtualBox-7.0.2-154219-OSX.dmg
==> Installing Cask virtualbox
==> Running installer for virtualbox; your password may be necessary.
Package installers may write to any location; options such as `--appdir` are ignored.
Password:
installer: Package name is Oracle VM VirtualBox
installer: choices changes file '/private/tmp/choices20221021-14368-qumemc.xml' applied
installer: Installing at base path /
installer: The install was successful.
==> Changing ownership of paths required by virtualbox; your password may be nec
🍺  virtualbox was successfully installed!
```

```console
❯ docker-machine create --driver virtualbox default
Creating CA: /Users/abhi3700/.docker/machine/certs/ca.pem
Creating client certificate: /Users/abhi3700/.docker/machine/certs/cert.pem
Running pre-create checks...
Error with pre-create check: "This computer doesn't have VT-X/AMD-v enabled. Enabling it in the BIOS is mandatory"
```

Check if docker supports virtualization:

```console
❯ sysctl kern.hv_support
kern.hv_support: 1
```

> This implies it supports virtualization, but need to be enabled in the BIOS.

TODO: How to enable virtualization in BIOS on Mac M1?

Once done, just follow the 2 steps from the [stack overflow reference](https://stackoverflow.com/a/49719638/6774636)

---

**Confirm docker is running**

- `$ docker version`
  > It should show the version of docker installed & client, server versions.

#### 2. On Linux VM [RECOMMENDED]

> It's better to alias docker with `sudo docker` to avoid permission issues.

```console
$ nano ~/.bashrc
```

alias docker='sudo docker'

```console
$ source ~/.bashrc
```

---

Switch to Linux VM

I am using Lima for this.

```console
❯ limactl start default
❯ lima
```

More related to [Lima](https://github.com/abhi3700/my_coding_toolkit/blob/main/vm_all.md#lima--install-ubuntu-arm-on-mac-arm).

---

**Install docker**

```console
$ sudo apt install docker.io
```

---

**Confirm docker is running**

- `$ sudo docker version`
  > It should show the version of docker installed & client, server versions.

---

**Docker login**:

```console
abhi3700@lima-default:/Users/abhi3700/F/coding/github_repos/My_Learning_NodeJSTS
$ sudo docker login
Login with your Docker ID to push and pull images from Docker Hub. If you don't have a Docker ID, head over to https://hub.docker.com to create one.
Username: abhi3700
Password:
WARNING! Your password will be stored unencrypted in /root/.docker/config.json.
Configure a credential helper to remove this warning. See
https://docs.docker.com/engine/reference/commandline/login/#credentials-store

Login Succeeded
```

Re-login

```console
abhi3700@lima-default:/Users/abhi3700/F/coding/github_repos/My_Learning_DevOps/docker/hello-docker
$ sudo docker login
Authenticating with existing credentials...
WARNING! Your password will be stored unencrypted in /root/.docker/config.json.
Configure a credential helper to remove this warning. See
https://docs.docker.com/engine/reference/commandline/login/#credentials-store

Login Succeeded
```

## Getting started

Prefer to do this inside Lima linux VM.

1. Create a project `hello-docker` directory via `$ mkdir hello-docker`
2. Add a file `app.js` inside `hello-docker` directory. Add the following code:

   ```js
   console.log("Hello Docker!");
   ```

3. Add a `Dockerfile` file inside `hello-docker` directory taking reference from this [YT video](https://www.youtube.com/watch?v=pTFZFxd4hOI).

   ```dockerfile
   FROM node:alpine
   COPY . /app
   WORKDIR /app
   CMD node app.js
   ```

4. Build the docker image via `$ docker build -t hello-docker .` inside lima linux VM terminal.
5. Check images via `$ docker images ls`. The image with latest

```console
abhi3700@lima-default:/Users/abhi3700/F/coding/github_repos/My_Learning_DevOps/docker/hello-docker$
$ sudo docker image ls
REPOSITORY     TAG       IMAGE ID       CREATED         SIZE
hello-docker   latest    dbbcb40a83b7   5 minutes ago   167MB
node           alpine    9bcdf8fa2b21   2 days ago      167MB
```

Hence, we can see that the image `hello-docker` is created.

All the images are stored in this location: `unix:///var/run/docker.sock: Get "http://%2Fvar%2Frun%2Fdocker.sock/v1.24/containers/json"`. Nothing is present in the repo.

6. Run the docker image via `$ docker run hello-docker` inside lima linux VM terminal.

```console
abhi3700@lima-default:/Users/abhi3700/F/coding/github_repos/My_Learning_DevOps/docker/hello-docker
$ sudo docker run hello-docker
Hello Docker!
```

7. Login using docker hub credentials via `$ sudo docker login` inside lima linux VM terminal.

8. Tag the docker image via `$ docker tag dbbcb40a83b7 abhi3700/hello-docker:latest` inside lima linux VM terminal. And then the image list would be like this:

```console
$ docker image ls
REPOSITORY              TAG            IMAGE ID       CREATED      SIZE
abhi3700/hello-docker   latest   dbbcb40a83b7   3 days ago   167MB
hello-docker            latest         dbbcb40a83b7   3 days ago   167MB
node                    alpine         9bcdf8fa2b21   5 days ago   167MB
```

9. Now, create a repository in the name of `hello-docker` in docker hub under username/account: `abhi3700`. Hence, the host would be [`docker.io/abhi3700/hello-docker`](https://hub.docker.com/r/abhi3700/hello-docker).
10. Publish the docker image to docker hub via `$ docker image push abhi3700/hello-docker:latest` inside lima linux VM terminal.

```console
The push refers to repository [docker.io/abhi3700/hello-docker]
c61658740dfd: Pushed
d0f090d9a0c6: Mounted from library/node
8678965b03c8: Mounted from library/node
7c50737eaab2: Mounted from library/node
5d3e392a13a0: Mounted from library/node
hello-docker: digest: sha256:7929aa35b2234697d88c7aa01ceef3d2cd8cf6d7a4579aaa76bd8f03afd5e5b0 size: 1365
```

11. In the docker hub, the image would be published & it looks like this:
    ![](../img/docker_hub_hello_docker_published.png)
12. Pull the docker image from docker hub via `$ docker pull abhi3700/hello-docker` inside any other linux VM terminal using [this](https://labs.play-with-docker.com/).

13. Now, the VM doesn't have node. So, just run the docker image via `$ docker run abhi3700/hello-docker`. So, you don't need any further tool installation or something but docker.

## Concepts

![](../img/container_vs_vm.png)

---

Running VMs on a physical hardware using Hypervisor.

![](../img/hypervisor.png)

---

We can run different machines (for testing a software with different versions) on a single physical hardware like this:

![](../img/multiple_vms.png)

---

Problems with VMs:
![](../img/vm_problems.png)

If a hardware has 8 GB RAM, then it has to be distributed into multiple VMs.

---

Benefit with Containers:

![](../img/container_benefit.png)

So, here in container we don't have to give a slice of hardware to each container. We can run 100s of containers on a single hardware, unlike VM where each takes a slice of hardware resources like RAM, CPU.

Technically a container is just a process running on a host machine.

---

Like in case of all hosts, we have a kernel, and all the containers share the kernel of the host machine.

![](../img/container_w_kernel.png)

> So, a linux container share linux kernel & a windows container share windows kernel.

> Now, a linux container can be run on a windows host machine, but a windows container can't be run on a linux host machine as linux machine doesn't have windows kernel and windows machine does have a linux kernel as WSL/WSL2.

But, in case of Mac, we don't have a linux kernel support. Hence, we have to use a linux VM to run linux containers. And Docker for Mac uses a lightweight linux VM like [Lima](https://github.com/lima-vm/lima) to run linux containers.
![](../img/kernels_on_hosts.png)

---

**DockerHub** is a registry of docker images. One can pull images from dockerhub and run them on their local machine.

---

**Docker Engine**

The core technology behind Docker. It is an open source software that runs on linux as a daemon that makes it possible to run containers on top of Linux kernel. It is responsible for the container lifecycle and isolation of physical resources (compute, memory, storage) that containers can access. The engine can run on a physical or a virtual machine, but it can only run on top of a Linux kernel i.e. any OS that is flavour of Linux. This is important to understand. Docker engine only runs on Linux.

## References

- [Docker Tutorial for Beginners | Programming with Mosh](https://www.youtube.com/watch?v=pTFZFxd4hOI)
