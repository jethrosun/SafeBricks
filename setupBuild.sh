#!/bin/bash

#sudo chown -R yangzhou:lambda-mpi-PG0 ~/NetBricks

curl https://sh.rustup.rs -sSf | sh  # Install rustup
source $HOME/.cargo/env
rustup install nightly-2019-12-20
rustup default nightly-2019-12-20

#dependencies for netbricks
sudo apt-get -y install libsctp-dev libssl-dev cmake llvm-3.9-dev libclang-3.9-dev clang-3.9 
