![](rust-banner.png)

# Server for Salamandra App

1. [Dependencies](#dependencies)
2. [Testing Locally](#testing-locally)
3. [Endpoints](#endpoints)

## Dependencies
 - Install rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

 - Install diesel
```bash
cargo install diesel_cli --no-default-features --features postgres 
```

 - Install docker
```bash
curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh
```

 - Other neccesary packages
```
sudo apt-get update
sudo apt-get install pkg-config libssl-dev libpq-dev docker-compose
```

## Testing locally
```bash
cd salamandra-server
chmod +x test_local
./test_local
```

## Endpoints
 - GET `/users/{user_id}` (Protected)
 - GET `/users/search?username=val`
