cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu

mkdir target/out

cp target/x86_64-apple-darwin/release/minecraft-server-installer target/out/installer-osx-x86_64
cp target/aarch64-apple-darwin/release/minecraft-server-installer target/out/installer-osx-aarch64
cp target/x86_64-unknown-linux-gnu/release/minecraft-server-installer target/out/installer-linux-x86_64
cp target/aarch64-unknown-linux-gnu/release/minecraft-server-installer target/out/installer-linux-aarch64
cp target/x86_64-pc-windows-gnu/release/minecraft-server-installer.exe target/out/installer-windows-x86_64.exe
