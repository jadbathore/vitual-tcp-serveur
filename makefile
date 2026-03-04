TARGET_WASI=wasm32-wasip2
TARGET_WASM=wasm32-unknown-unknown
IMAGE=virtual-front-end
CONTAINER_NAME=vfs-test
VFS=../test/
ADDRESS="localhost:8080"

build-lib:
	cd communUtilsHandler && cargo build --release \
	&& cd deriveUtils && cargo build --release


build-wasi:
	cd fsHandlerWasi && cargo component build --target $(TARGET_WASI) --release --target-dir ../virtualFile/target

build-host:
	cd virtualFile && VFS_DIR=$(VFS) ADDRESS=$(ADDRESS) RUST_BACKTRACE=1 cargo build --release

run:
	cd virtualFile && VFS_DIR=$(VFS) ADDRESS=$(ADDRESS) RUST_BACKTRACE=1 cargo run 

build-all: build-lib build-wasi run

clean-all:
	cd fsHandlerWasi && cargo clean 
	cd virtualFile && cargo clean 

build-image: 
	docker buildx build -f ./dockerfile . -t $(IMAGE) --platform linux/arm64 

test-container:
	docker compose --project-name $(CONTAINER_NAME) build;
	docker compose --project-name $(CONTAINER_NAME) up;
	

test-image:
	docker run -it  $(IMAGE) /bin/sh
bind:
	cd fsHandlerWasi && cargo component bindings 


build-hash:
	cd snapshot-hash/ && cargo run