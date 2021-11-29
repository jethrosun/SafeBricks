#!/bin/bash
source ./config.sh
set -e

BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
echo $BASE_DIR
BUILD_SCRIPT=$( basename "$0" )

if [[ -z ${CARGO_INCREMENTAL} ]] || [[ $CARGO_INCREMENTAL = false ]] || [[ $CARGO_INCREMENTAL = 0 ]]; then
    export CARGO_INCREMENTAL="CARGO_INCREMENTAL=0 "
fi

if [[ -z ${RUST_BACKTRACE} ]] || [[ RUST_BACKTRACE = true ]] || [[ RUST_BACKTRACE = 1 ]]; then
    export RUST_BACKTRACE="RUST_BACKTRACE=1 "
fi

echo "Current Cargo Incremental Setting: ${CARGO_INCREMENTAL}"
echo "Current Rust Backtrace Setting: ${RUST_BACKTRACE}"

DPDK_VER=17.08.1
DPDK_HOME="$HOME/dev/tools/dpdk-stable-${DPDK_VER}"
DPDK_LD_PATH="${DPDK_HOME}/build/lib"
DPDK_CONFIG_FILE=${DPDK_CONFIG_FILE-"${DPDK_HOME}/config/common_linuxapp"}

export RTE_SDK=$HOME/dev/tools/dpdk-stable-17.08.1 # for instance.

NATIVE_LIB_PATH="${BASE_DIR}/native"
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

# clang 3.8
# export PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/bin/:$PATH
# export LLVM_CONFIG_PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/bin/llvm-config
# export LD_LIBRARY_PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/lib

native () {
    make -j $proc -C $BASE_DIR/native
    make -C $BASE_DIR/native install
}

native

# Build custom runner
pushd dpdkIO
if [ "$MODE" == "debug" ]; then
    cargo +nightly-2019-12-20 build
else
    cargo +nightly-2019-12-20 build --release
fi
popd

# Build custom runner
pushd sgx-runner
if [ "$MODE" == "debug" ]; then
    cargo +nightly-2019-12-20 build
else
    cargo +nightly-2019-12-20 build --release
fi
popd

# export HYPERSCAN_ROOT=/usr/local
# for TASK in dpi-hs
# for TASK in acl-fw dpi lpm macswap maglev monitoring nat-tcp-v4 acl-fw-ipsec dpi-ipsec lpm-ipsec macswap-ipsec maglev-ipsec monitoring-ipsec nat-tcp-v4-ipsec acl-fw-ipsec-sha dpi-ipsec-sha lpm-ipsec-sha macswap-ipsec-sha maglev-ipsec-sha monitoring-ipsec-sha nat-tcp-v4-ipsec-sha
for TASK in acl-fw 
do 

	# Build enclave APP
	pushd examples/$TASK
	if [ "$MODE" == "debug" ]; then
	    cargo +nightly-2019-12-20 build --target=x86_64-fortanix-unknown-sgx
	else
	    cargo +nightly-2019-12-20 build --target=x86_64-fortanix-unknown-sgx --release
	fi
	popd

	# Convert the APP
	if [ "$MODE" == "debug" ]; then # 2a
	    ftxsgx-elf2sgxs target/x86_64-fortanix-unknown-sgx/$MODE/$TASK --heap-size 0x5d80000 --stack-size 0x5d80000 --threads 2 --debug
	else
	    ftxsgx-elf2sgxs target/x86_64-fortanix-unknown-sgx/$MODE/$TASK --heap-size 0x5d80000 --stack-size 0x5d80000 --threads 2
	fi
done
