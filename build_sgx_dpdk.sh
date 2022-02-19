#!/bin/bash
source ./config.sh
set -e

BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
echo $BASE_DIR
BUILD_SCRIPT=$( basename "$0" )

CARGO_PATH="$HOME/.cargo/bin/cargo"
CARGO_LOC=$(which cargo || true)
export CARGO=${CARGO_PATH-"${CARGO_LOC}"}
if [ -z "${CARGO}" ] || [ ! -e "${CARGO}" ]; then
    echo "Could not find a preinstalled Cargo in PATH. Set CARGO_PATH if necessary."
    exit 1
fi
echo "Using Cargo from ${CARGO}"

# RUST_TEST="${TOOLS_BASE}/bin/rustc"
# RUST_DOWNLOAD_PATH="${EXT_BASE}/rust"

export RUSTFLAGS="-C target-cpu=native"

# if [[ -z ${CARGO_INCREMENTAL} ]] || [[ $CARGO_INCREMENTAL = false ]] || [[ $CARGO_INCREMENTAL = 0 ]]; then
#     export CARGO_INCREMENTAL="CARGO_INCREMENTAL=0 "
# fi
#
# if [[ -z ${RUST_BACKTRACE} ]] || [[ RUST_BACKTRACE = true ]] || [[ RUST_BACKTRACE = 1 ]]; then
#     export RUST_BACKTRACE="RUST_BACKTRACE=1 "
# fi
#
# echo "Current Cargo Incremental Setting: ${CARGO_INCREMENTAL}"
# echo "Current Rust Backtrace Setting: ${RUST_BACKTRACE}"
#
# # just enforce it
# export CARGO_INCREMENTAL="CARGO_INCREMENTAL=0 "
# export RUST_BACKTRACE="RUST_BACKTRACE=0 "
#
# # We fix the Cargo toolchain
declare TOOLCHAIN=nightly-2020-05-30-x86_64-unknown-linux-gnu
# declare SGX_TOOLCHAIN=nightly-2020-05-30-x86_64-fortanix-unknown-sgx

DPDK_VER=17.08.1
DPDK_HOME="$HOME/dev/tools/dpdk-stable-${DPDK_VER}"
DPDK_LD_PATH="${DPDK_HOME}/build/lib"
DPDK_CONFIG_FILE=${DPDK_CONFIG_FILE-"${DPDK_HOME}/config/common_linuxapp"}

export RTE_SDK=$HOME/dev/tools/dpdk-stable-17.08.1 # for instance.

NATIVE_LIB_PATH="${BASE_DIR}/native"
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

TARGET_DIR="${HOME}/data/cargo-target/x86_64-fortanix-unknown-sgx"
# RELEASE_TARGET_DIR="${HOME}/data/cargo-target/${MODE}release"
# DEBUG_TARGET_DIR="${HOME}/data/cargo-target/${MODE}debug"

# NIGHTLY=nightly-2021-01-20
NIGHTLY=nightly-2020-05-30

# clang 3.8
# export PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/bin/:$PATH
# export LLVM_CONFIG_PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/bin/llvm-config
# export LD_LIBRARY_PATH=$HOME/dev/clang-3.8/clang+llvm-3.8.1-x86_64-linux-gnu-ubuntu-16.04/lib

native () {
    make -j $proc -C $BASE_DIR/native
    make -C $BASE_DIR/native install
}

native

# # Build custom runner
# pushd dpdkIO
# if [ "$MODE" == "debug" ]; then
#     ${CARGO} +${TOOLCHAIN} build
# else
#     ${CARGO} +${TOOLCHAIN} build --release
# fi
# popd
#
# # Build custom runner
# pushd sgx-runner
# if [ "$MODE" == "debug" ]; then
#     ${CARGO} +${TOOLCHAIN} build
# else
#     ${CARGO} +${TOOLCHAIN} build --release
# fi
# popd

# for TASK in acl-fw dpi lpm macswap maglev monitoring nat-tcp-v4 acl-fw-ipsec dpi-ipsec lpm-ipsec macswap-ipsec maglev-ipsec monitoring-ipsec nat-tcp-v4-ipsec acl-fw-ipsec-sha dpi-ipsec-sha lpm-ipsec-sha macswap-ipsec-sha maglev-ipsec-sha monitoring-ipsec-sha nat-tcp-v4-ipsec-sha
for TASK in traversal #forward
do
    # Build enclave APP
    pushd examples/$TASK
    if [ "$MODE" == "debug" ]; then
	${CARGO} +${NIGHTLY} build --target=x86_64-fortanix-unknown-sgx
    else
	${CARGO} +${NIGHTLY} build --target=x86_64-fortanix-unknown-sgx --release
    fi
    popd

    # Convert the APP
    if [ "$MODE" == "debug" ]; then # 2a
	ftxsgx-elf2sgxs ${TARGET_DIR}/$MODE/$TASK --heap-size 0x5d80000 --stack-size 0x5d80000 --threads 2 --debug
    else
	ftxsgx-elf2sgxs ${TARGET_DIR}/$MODE/$TASK --heap-size 0x5d80000 --stack-size 0x5d80000 --threads 2
    fi
done
