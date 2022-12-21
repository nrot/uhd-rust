#!/usr/bin/env bash

export DEBIAN_FRONTEND=noninteractive

apt-get -y update

apt-get -y install autoconf automake build-essential ccache cmake cpufrequtils doxygen ethtool \
g++ git inetutils-tools libboost-all-dev libncurses5 libncurses5-dev libusb-1.0-0 libusb-1.0-0-dev \
libusb-dev python3-dev python3-mako python3-numpy python3-requests python3-scipy python3-setuptools \
python3-ruamel.yaml 

# apt-get -y install libuhd-dev
# cd src/uhd-rust/uhd-sys/uhd/host
git clone https://github.com/EttusResearch/uhd.git

cd uhd/host

mkdir build 
cd build 
cmake  ../ 
#-DENABLE_STATIC_LIBS=ON
make --jobs=$(nproc)
make install
