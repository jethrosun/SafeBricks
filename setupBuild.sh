#!/bin/bash

#sudo chown -R yangzhou:lambda-mpi-PG0 ~/NetBricks


#!/bin/bash

#sudo chown -R yangzhou:lambda-mpi-PG0 ~/NetBricks


# echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
# curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

# curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
# sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu bionic stable"
sudo apt-get update

#dependencies for netbricks
sudo apt install silversearcher-ag -y
sudo apt-get -q -y install clang libclang-dev libsctp-dev libssl-dev cmake pkg-config zlib1g-dev
sudo apt-get -q -y install pkg-config libssl-dev protobuf-compiler
sudo apt-get -q -y install build-essential ocaml automake autoconf libtool wget python libssl-dev
sudo apt-get -q -y install libssl-dev libcurl4-openssl-dev protobuf-compiler libprotobuf-dev debhelper cmake
sudo apt-get -q -y install intel-sgx-dkms
sudo apt-get -y install libsctp-dev libssl-dev cmake llvm-3.9-dev libclang-3.9-dev clang-3.9 gcc-multilib
sudo apt install -q -y docker-ce libsgx-enclave-common sgx-aesm-service libsgx-aesm-launch-plugin


if [ -e $HOME/.cargo/ ]; then
        echo "Passing, Rust already exists.."
        source $HOME/.cargo/env
        rustup default nightly-2020-05-30-x86_64-unknown-linux-gnu
else
        # setup rust
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
        rustup toolchain install 2020-05-30-x86_64-unknown-linux-gnu --allow-downgrade --profile minimal --component rust-src rustfmt
        source $HOME/.cargo/env
fi

rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly-2020-05-30


sudo usermod -aG docker ${USER}
sudo su - ${USER}

sudo systemctl status docker

docker run --detach --restart always --device /dev/isgx --volume /var/run/aesmd:/var/run/aesmd --name aesmd fortanix/aesmd

echo "Now install fortanix-sgx-tools sgxs-tools"

