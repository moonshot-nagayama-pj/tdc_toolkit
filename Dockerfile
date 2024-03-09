FROM --platform=linux/amd64 mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# This Dockerfile is configured for development purposes, not production.
# The apt cache is retained to facilitate the addition of more tools via apt during development.
RUN apt-get update -y && apt-get install -y software-properties-common && \
 add-apt-repository ppa:deadsnakes/ppa && \
 apt-get update -y && \
 apt-get install -y python3.12-dev python3.12-venv python3.12-distutils  \
 llvm-dev libclang-dev clang lld file git \
 libusb-1.0-0-dev libusb-1.0-0 usbutils curl tig \
 libffi-dev pkg-config libreadline-dev vim  && \
 update-alternatives --install /usr/bin/python python /usr/bin/python3.12 1

USER vscode
WORKDIR /home/vscode
RUN wget https://static.rust-lang.org/rustup/archive/1.26.0/x86_64-unknown-linux-gnu/rustup-init && \
 chmod +x rustup-init && \
 ./rustup-init -y

RUN curl -sSf https://rye-up.com/get | RYE_INSTALL_OPTION="-y" bash
RUN echo "source $HOME/.rye/env" >> $HOME/.profile
