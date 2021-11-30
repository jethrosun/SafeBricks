[![Build Status](https://travis-ci.org/williamofockham/NetBricks.svg?branch=master)](https://travis-ci.org/williamofockham/NetBricks)

[NetBricks](http://netbricks.io/) is a Rust based framework for NFV development. Please refer to the
[paper](https://people.eecs.berkeley.edu/~apanda/assets/papers/osdi16.pdf) for information
about the architecture and design. Currently NetBricks requires a relatively modern Linux version.

Up and Running
----------------

NetBricks can built within a Docker container. In this case, you do not need to
install any of the dependencies, and the final product can be run the same.
However to run NetBricks you still need to be on a machine or VM that is
correctly configured to run [DPDK](https://www.dpdk.org/) (version 17.08.1).

If you use our vagrant-based Developer environment, all DPDK configuration is
done for you. We also include the [MoonGen](//github.com/williamofockham/MoonGen) traffic generator and the
[Containernet](//github.com/containernet/containernet) virtual network simulator for testing and development.

## Setting up your local Ubuntu-16.04 environment

1. Clone our [utils](//github.com/YangZhou1997/utils) and [moonGen](//github.com/YangZhou1997/MoonGen)
repositories into the same parent directory.
```shell
host$ for repo in utils moonGen NetBricks; do \
git clone --recurse-submodules git@github.com:YangZhou1997/${repo}.git; \
done
```

2. Update and install packages. Any kernel should be generally okay -- we have
tested on 4.4.0-131-generic, 4.4.0-142-generic, 4.4.0-145-generic, and
4.15.0-15-generic. NOTE: you have to do the kernel upgrade to get module
`uio_pci_generic`

```shell
host$ sudo bash ../utils/vm-kernel-upgrade.sh #require rebooting
host$ sudo shutdown -r now
host$ sudo bash ../utils/vm-setup.sh
```

3. After step 2, you machine meets the basic requirements of running NetBricks. Now you need to build and bind DPDK using [setupDpdk.sh](./setupDpdk.sh). 
```shell
host$ mkdir $HOME/dev/trash
host$ mkdir $HOME/dev/tools
host$ ./setupDpdk.sh # in user
```

## Developing in local Ubuntu-16.04 environment

1. Make sure that you have gone though step 1-3 of last section successfully. Current version of NetBricks will read some DPDK lib from /opt/dpdk/build/ during runtime, you need to copy include/ and lib/ directory from $RTE_SDK/build to /opt/dpdk/build/. Note that soft links need to be considered carefully. We provide [setupDpdkCopy.sh](./setupDpdkCopy.sh) for that (actually, `setupDpdkCopy.sh` has been executed in `setupDpdk.sh`): 
```shell
host$ ./setupDpdkCopy.sh # in user
```

2. As far as I know, NetBricks assumes you are root when running it. So you need to switch to root now. 
```shell
# NOTE: this is a real bad idea!
host$ sudo su
root$ ./setupBuild.sh 

# Get fortanix stuff
echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu bionic stable"
sudo apt-get update

# You should still run everything in user mode
host$ ./setupBuild.sh
```

[setupBuild.sh](./setupBuild.sh) will install the rust nightly, clang, and etc for running NetBricks. 

This NetBricks codebase works on rust nightly-2020-05-30. You can override the rust version in current directory to nightly-2020-05-30 by:
```shell
rustup install nightly-2020-05-30
rustup override set nightly-2020-05-30
```

### Install Rust SGX tools

```shell
...
rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly-2020-05-30
...
cd rust-sgx/fortanix-sgx-tools
cargo +nightly-2020-05-30 b --release
cargo +nightly-2020-05-30 install --path . 
...
cd ../sgxs-tools
cargo +nightly-2020-05-30 b --release
cargo +nightly-2020-05-30 install --path . 
echo >> ~/.cargo/config -e '[target.x86_64-fortanix-unknown-sgx]\nrunner = "ftxsgx-runner-cargo"'
```

3. After step 2, you need to set `RTE_SDK` to the dpdk directory, and load cargo environment. Then you'll be able to compile and test NetBricks:
```shell
host$ export RTE_SDK=$HOME/dev/tools/dpdk-stable-17.08.1 # for instance.
host$ source $HOME/.cargo/env
host$ make build
...
host$ make test
...
```

We also provide some commands that might be helpful when dealing with DPDK hugepages in [setupHuge.sh](./setupHuge.sh).

**Note**: when you switch between local deployment and container deployment, you need to `sudo make clean` to rebuild the dependencies in native/ (especially .make.dep).  

**Note**: if you find numerous error printed during `make build`, it is caused by the bindgen (generating rust binding for dpdk); you can solve it by deleting `~/tools/dpdk-stable-17.08.1` and run `./setupDpdk.sh`. The specific reason is that you might download my hacked version of dpdk, which will fail the bindgen binding. 

## Enabling SGX if `Software Controlled` set

Clone linux-sgx and build in your home directory:
```shell
cd ~/dev && git clone git@github.com:intel/linux-sgx.git
sudo apt-get -y install build-essential ocaml automake autoconf libtool wget python libssl-dev
sudo apt-get -y install libssl-dev libcurl4-openssl-dev protobuf-compiler libprotobuf-dev debhelper cmake
cd linux-sgx
./download_prebuilt.sh
make -j16
```

Enable SGX in your machine which set **Software Controlled**: 
```shell
gcc enable_sgx.cpp -o enable_sgx -L$HOME/dev/linux-sgx/sdk/libcapable/linux -lsgx_capable -I$HOME/dev/linux-sgx/common/inc/
sudo LD_LIBRARY_PATH=$HOME/dev/linux-sgx/sdk/libcapable/linux ./enable_sgx
```

From https://github.com/intel/linux-sgx/issues/354: 
is_sgx_capable has to come back a 1 to be able to be enabled.
If so, then status should come back a 1 also, which means "SGX_DISABLED_REBOOT_REQUIRED". Once you reboot, you should get a 0 back for the second.
Yes! Zero means "SGX_ENABLED". :-) 

Install SGX driver and Fortanix EDP following: https://edp.fortanix.com/docs/installation/guide/. 

**Note**: when you encountered some `No such file or directory` error, try to reinstall SGX driver and Fortanix EDP. 
