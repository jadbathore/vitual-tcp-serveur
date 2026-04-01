FROM --platform=$BUILDPLATFORM rust:latest 

COPY . /tcp-server

RUN apt-get update && apt-get install -y \
make;
RUN cargo install cargo-component 
RUN rustup target add wasm32-wasip2
RUN cd /tcp-server && make build-all VM-CONTEXT=1
EXPOSE 8080