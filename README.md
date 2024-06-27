# solana_Cargo
=============== Install Step
1. The easiest way to get Cargo is to install the current stable release of Rust by using rustup. 
    Installing Rust using rustup will also install cargo.
    On Linux and macOS systems, this is done as follows:    
        curl https://sh.rustup.rs -sSf | sh

2. Install the Solana release v1.18.17 on your machine by running:
        sh -c "$(curl -sSfL https://release.solana.com/v1.18.17/install)"

3. Install avm using Cargo. Note this will replace your anchor binary if you had one installed.
        cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
    On Linux systems you may need to install additional dependencies if cargo install fails. E.g. on Ubuntu:
        sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev
    Install the latest version of the CLI using avm, and then set it to be the version to use.
        avm install 0.30.0

4. Initial and Deployment
    - Check Balance
        solana balance --url https://api.devnet.solana.com

=============== Initial Env
1. cargo new --lib solana_Cargo
2. cd solana_Cargo
2. solana-keygen new --no-bip39-passphrase -o ./wallets/id.json
4. solana config set --url https://api.devnet.solana.com -k ./wallets/id.json
5. cargo build-bpf --manifest-path=./Cargo.toml --bpf-out-dir=dist/program
6. solana program deploy dist/program/rev_gold.so
7. - Necessary balance = 4.01788904 - 2.1183252 = 1.9