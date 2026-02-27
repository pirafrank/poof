# APT/YUM package repositories

poof is available via native package managers for Debian/Ubuntu-based systems (APT)
and Red Hat-based systems (DNF/YUM).

Packages are built for **amd64** and **arm64** architectures.

## Debian and Ubuntu (APT)

Supported distributions:

- Debian 9 (stretch) and newer
- Ubuntu 16.04 (xenial) and newer

## Install

```sh
curl -fsSL https://poof-pkgs.fpira.com/apt/gpg.pub \
  | sudo gpg --dearmor -o /usr/share/keyrings/poof.gpg
echo "deb [signed-by=/usr/share/keyrings/poof.gpg] https://poof-pkgs.fpira.com/apt stable main" \
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

---

## RHEL, CentOS, Amazon Linux (DNF/YUM)

Supported distributions:

- Fedora 24
- RHEL 8 / CentOS 8 / CentOS Stream 8
- RHEL 9 / CentOS Stream 9
- Amazon Linux 2
- Amazon Linux 2023

### Install

**RHEL 9 / CentOS Stream 9:**

```sh
sudo rpm --import https://poof-pkgs.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://poof-pkgs.fpira.com/yum/el9/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://poof-pkgs.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
```

**RHEL 8 / CentOS 8 / CentOS Stream 8:**

```sh
sudo rpm --import https://poof-pkgs.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://poof-pkgs.fpira.com/yum/el8/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://poof-pkgs.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
```

**Amazon Linux 2023:**

```sh
sudo rpm --import https://poof-pkgs.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://poof-pkgs.fpira.com/yum/amzn2023/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://poof-pkgs.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
```

**Amazon Linux 2:**

```sh
sudo rpm --import https://poof-pkgs.fpira.com/yum/gpg.pub
sudo tee /etc/yum.repos.d/poof.repo << EOF
[poof]
name=poof
baseurl=https://poof-pkgs.fpira.com/yum/amzn2/$(uname -m)/
enabled=1
gpgcheck=1
gpgkey=https://poof-pkgs.fpira.com/yum/gpg.pub
EOF
sudo dnf install poof
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
