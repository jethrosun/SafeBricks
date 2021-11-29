#!/bin/bash
set -e

#sudo bash ../utils/vm-kernel-upgrade.sh
#require rebooting

#sudo bash ../utils/vm-setup.sh

DPDK_HOME=~/dev/tools/dpdk-stable-17.08.1

# add -DHG_MON=1 if you want dpdk to print memzone info.
CFLAGS="-g3 -Wno-error=maybe-uninitialized -fPIC"

# clang 3.8
# export PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/bin/:$PATH
# export LLVM_CONFIG_PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/bin/llvm-config
# export LD_LIBRARY_PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/lib

sudo apt-get -y install build-essential ca-certificates curl \
    libnuma-dev libpcap-dev xz-utils llvm-3.9-dev libclang-3.9-dev clang-3.9 cmake

# This is used when you want to monitor dpdk hugepage usage during runtime.
build_dpdk_hugepage_mon () {
    if [ ! -d "dpdk-stable-17.08.1" ]; then
        curl -sSf https://fast.dpdk.org/rel/dpdk-17.08.1.tar.xz | tar -xJv
    elif [ ! -d "dpdk-stable-17.08.1/.git" ]; then
        sudo rm -rf dpdk-stable-17.08.1/
        git clone git@github.com:YangZhou1997/dpdk-stable-17.08.1.git
    else
        echo "Rebuild dpdk!"
    fi
}

# This is used when you normally want to rebuild dpdk in case that you made some modification.
build_dpdk_normal () {
    if [ ! -d "dpdk-stable-17.08.1" ]; then
        curl -sSf https://fast.dpdk.org/rel/dpdk-17.08.1.tar.xz | tar -xJv
    else
        echo "Just build!"
    fi
}

cd ~/dev/tools
# build_dpdk_hugepage_mon
build_dpdk_normal


cp ~/dev/utils/dpdk/common_linuxapp-17.08 $DPDK_HOME/config/common_linuxapp

cd $DPDK_HOME

make clean | true
make config T=x86_64-native-linuxapp-gcc EXTRA_CFLAGS="${CFLAGS}"
make -j16 EXTRA_CFLAGS="${CFLAGS}"
sudo make install

sudo insmod $DPDK_HOME/build/kmod/igb_uio.ko

sudo $DPDK_HOME/usertools/dpdk-devbind.py --force -b igb_uio 0000:02:00.0 0000:02:00.1 0000:02:00.2 0000:02:00.3

bash ~/dev/SafeBricks/setupDpdkCopy.sh

# hugepages setup on numa node
# echo 2048 | sudo tee /sys/devices/system/node/node0/hugepages/hugepages-2048kB/nr_hugepages
# echo 2048 | sudo tee /sys/devices/system/node/node1/hugepages/hugepages-2048kB/nr_hugepages

echo "please rebuild SafeBricks to make dpdk changes valid"
