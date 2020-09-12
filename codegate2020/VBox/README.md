# VBox (pwn)

## Description

VBox, small, but powerful sandbox with virtualization.
Escape the box and get the shell.

### Compile
```sh
$ curl https://sh.rustup.rs -sSf | sh  # installing the cargo
$ RUSTFLAGS='-C link-arg=-s' cargo build --release
$ cp target/release/vbox .
```

### Build Docker
```sh
docker build --build-arg KVM_ROOT=$(getent group kvm | cut -d: -f3) -t vbox .
```

### Run Docker
```sh
docker run --name vbox --privileged -d -p1000:1000 vbox
```

### WARNING
KVM should be enabled on host machine.
