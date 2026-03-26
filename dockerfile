FROM --platform=$BUILDPLATFORM rust:latest 

COPY . /tcp-server


RUN cargo install cargo-component 
RUN rustup target add wasm32-wasip2
RUN cd tcp-server/fsHandlerWasi && cargo component build --target wasm32-wasip2 --release --target-dir ../virtualFile/target
EXPOSE 8080
# FROM ubuntu:latest

# COPY . /bin.sh