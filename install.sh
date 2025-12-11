cargo install --path .
cd gos-std
cargo build --release
sudo cp target/release/libgos_std.a /usr/local/lib/libgos.a
sudo cp -r gos/ /usr/local
