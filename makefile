TARGET_WASI=wasm32-wasip2
TARGET_WASM=wasm32-unknown-unknown
IMAGE=virtual-front-end
build-wasi:
	cd fsHandlerWasi && cargo component build --target $(TARGET_WASI) --release --target-dir ../virtualFile/target

test-host:
	cd virtualFile && VFS_DIR="../test/" RUST_BACKTRACE=1 cargo run 

build-all: build-wasi test-host

clean-all:
	cd fsHandlerWasi && cargo clean 
	cd virtualFile && cargo clean 

build-image: 
	docker buildx build -f ./docker_vfs/dockerfile . -t $(IMAGE) --platform linux/arm64 

test-image:
	docker run -it  $(IMAGE) /bin/sh
bind:
	cd fsHandlerWasi && cargo component bindings 