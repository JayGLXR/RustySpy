FROM rust:latest

RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    g++-mingw-w64-x86-64

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /usr/src/rustyspy
COPY . .

RUN cargo build --target x86_64-pc-windows-gnu --release

CMD ["cp", "target/x86_64-pc-windows-gnu/release/rustyspy.exe", "/output/"] 