# APT/YUM/APK package repositories

poof is available via native package managers for popular Linux distributions:

[Debian/Ubuntu-based systems (APT)](#debian-and-ubuntu-apt) |
[Red Hat-based systems (DNF/YUM)](#fedora-rhel-centos-amazon-linux) |
[Alpine Linux (APK)](#alpine-linux-apk)

## Automation

Generation, packaging and signing of `.deb`, `.rpm`, `.apk` files is [handled
automatically](https://github.com/pirafrank/packages).

## Debian and Ubuntu (APT)

### Supported architectures

- `amd64` (`x86_64`)
- `arm64` (`aarch64`)
- `armhf` (`armv7`)
- `i386` (`i686`)
- `riscv64` (`riscv64gc`)

### Supported distributions

- Debian 9 (stretch) and newer
- Ubuntu 16.04 (xenial) and newer

### Install

```sh
curl -fsSL https://pkg.fpira.com/apt/gpg.pub \
  | sudo gpg --dearmor -o /usr/share/keyrings/poof.gpg
echo "deb [signed-by=/usr/share/keyrings/poof.gpg] https://pkg.fpira.com/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/poof.list
sudo apt update && sudo apt install poof
```

### Uninstall

Uninstall the package by running:

```sh
sudo apt remove poof
```

To also remove the repository:

```sh
sudo rm /etc/apt/sources.list.d/poof.list /usr/share/keyrings/poof.gpg
sudo apt update
```

## Fedora, RHEL, CentOS, Amazon Linux

### Supported architectures

- `x86_64`
- `aarch64`

### Supported distributions

- Fedora 24 and newer
- RHEL 8 / CentOS 8 / CentOS Stream 8
- RHEL 9 / CentOS Stream 9
- Amazon Linux 2
- Amazon Linux 2023

### Install

**RHEL 9 / CentOS Stream 9 / Fedora 36+:**

```sh
sudo rpm --import https://pkg.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://pkg.fpira.com/yum/el9/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://pkg.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
```

**RHEL 8 / CentOS 8 / CentOS Stream 8 / Fedora 24-35:**

```sh
sudo rpm --import https://pkg.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://pkg.fpira.com/yum/el8/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://pkg.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
```

**Amazon Linux 2023:**

```sh
sudo rpm --import https://pkg.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://pkg.fpira.com/yum/amzn2023/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://pkg.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
```

**Amazon Linux 2:**

```sh
sudo rpm --import https://pkg.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://pkg.fpira.com/yum/amzn2/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://pkg.fpira.com/yum/gpg.pub
EOF
sudo yum install poof
```

### Uninstall

```sh
# on newer systems:
sudo dnf remove poof
# or on older distributions:
sudo yum remove poof
```

To also remove the repository:

```sh
sudo rm /etc/yum.repos.d/poof.repo
```

## Alpine Linux (APK)

### Supported architectures

- `x86_64`
- `aarch64`
- `armv7`
- `riscv64`

### Supported versions

We support the latest four stable Alpine in the repository.

- Alpine 3.23
- Alpine 3.22
- Alpine 3.21
- Alpine 3.20

You can always use other install methods (e.g. install script, or manual download
from [releases](https://github.com/pirafrank/poof/releases/latest)) to bring musl
builds to older Alpine versions.

### Install

```sh
wget -q -O /etc/apk/keys/signing-key.rsa.pub \
  https://pkg.fpira.com/apk/signing-key.rsa.pub
ALPINE_VERSION=$(cat /etc/alpine-release | cut -d. -f1,2)
echo "https://pkg.fpira.com/apk/v${ALPINE_VERSION}" \
  >> /etc/apk/repositories
apk update && apk add poof
```

### Uninstall

```sh
apk del poof
```

To also remove the repository:

```sh
sed -i '/pkg\.fpira\.com\/apk/d' /etc/apk/repositories
rm -f /etc/apk/keys/signing-key.rsa.pub
```
