FROM ubuntu

# basic requirements
RUN apt-get update && apt-get install -y \
    curl wget apt-transport-https software-properties-common

# nushell
RUN wget -qO- https://apt.fury.io/nushell/gpg.key | gpg --dearmor -o /etc/apt/keyrings/fury-nushell.gpg \
      && echo "deb [signed-by=/etc/apt/keyrings/fury-nushell.gpg] https://apt.fury.io/nushell/ /" | tee /etc/apt/sources.list.d/fury-nushell.list

# powershell
RUN . /etc/os-release \
      && wget -q https://packages.microsoft.com/config/ubuntu/$VERSION_ID/packages-microsoft-prod.deb \
      && dpkg -i packages-microsoft-prod.deb \
      && rm packages-microsoft-prod.deb


# refresh apt and install shells
RUN apt-get update && apt-get install -y \
    zsh \
    fish \
    elvish \
    nushell \
    powershell \
    xonsh

# add user and change password
RUN echo 'add user and change password' \
  && useradd -Um -d /home/user -s /bin/bash user \
  && echo "root:root" | chpasswd \
  && echo "user:user" | chpasswd

# run as user
USER user
WORKDIR /home/user

# install poof and add
RUN curl -fsSL https://raw.githubusercontent.com/pirafrank/poof/main/install.sh | sh \
  && echo 'export PATH="${HOME}/.local/bin:$PATH"' >> ~/.bashrc \
  && echo "poof installed successfully" \
  && echo "poof install directory: ${HOME}/.local/bin"

# enable poof and check version
ENV PATH="/home/user/.local/bin:${PATH}"
RUN poof version \
  && poof enable --shell bash \
  && poof enable --shell zsh \
  && poof enable --shell fish \
  && poof enable --shell elvish \
  && poof enable --shell nushell \
  && poof enable --shell powershell \
  && poof enable --shell xonsh
