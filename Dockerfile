FROM --platform=linux/amd64 mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# This Dockerfile is configured for development purposes, not production.
# The apt cache is retained to facilitate the addition of more tools via apt during development.
RUN apt update -y && apt install -y \
 git python3.11 python3-pip python3.11-venv \
 llvm-dev libclang-dev clang lld file \
 libusb-1.0-0-dev libusb-1.0-0 usbutils curl tig
USER vscode
WORKDIR /home/vscode
RUN wget https://static.rust-lang.org/rustup/archive/1.26.0/x86_64-unknown-linux-gnu/rustup-init && \
 chmod +x rustup-init && \
 ./rustup-init -y

