FROM --platform=linux/amd64 ubuntu:latest

RUN apt update -y && apt install -y git python3.11 python3-pip python3.11-venv libusb-1.0-0-dev libusb-1.0-0 usbutils curl tig

