
#-----------target-----------------
TARGET_WASI=wasm32-wasip2
TARGET_WASM=wasm32-unknown-unknown
#-----------Docker-----------------
IMAGE=virtual-front-end
CONTAINER_NAME=vfs-test
#-----------utils-----------------
VFS=../test/
ADDRESS="localhost:8080"
ifeq ($(VM-CONTEXT),1)
	ADDRESS="0.0.0.0:8080"
endif
#-----------pre-cmd-----------------
HOST_ARGS=cd virtualFile && VFS_DIR=$(VFS) ADDRESS=$(ADDRESS) RUST_BACKTRACE=1 
#-----------build-------------------
build-lib:
	cd communUtilsHandler && cargo build --release \
	&& cd derive_utils && cargo build --release

build-wasi:
	cd fsHandlerWasi && cargo component build \
	--target $(TARGET_WASI) \
	--release --target-dir ../virtualFile/target

build-host:
	$(HOST_ARGS)cargo build --release --features client

build-deamon:
	$(HOST_ARGS)cargo build --release --features deamon \
	--target-dir target/deamon

build-all: build-lib build-wasi build-host

init: build-lib build-wasi run
#-----------binding-----------------
bind:
	cd fsHandlerWasi && cargo component bindings 
#-----------run---------------------
run-client:
	$(HOST_ARGS)cargo run --features client

run-deamon:
	$(HOST_ARGS)cargo run --features deamon

no-compile-run-client:
	$(HOST_ARGS)./target/release/virtualFile

no-compile-run-deamon: 
	$(HOST_ARGS)./target/deamon/release/virtualFile
#-------------clean-------------------
clean-host:
	cd virtualFile && cargo clean 

clean-wasi: 
	cd fsHandlerWasi && cargo clean 

clean-lib:
	cd communUtilsHandler && cargo clean \
	&& cd derive_utils && cargo clean
clean-all:clean-host clean-wasi clean-lib
#-------------docker-------------------
build-image: 
	docker buildx build -f ./dockerfile . -t $(IMAGE) --platform linux/arm64 
test-container:
	docker compose --project-name $(CONTAINER_NAME) build;
	docker compose --project-name $(CONTAINER_NAME) up;
	
test-image:
	docker run -it  $(IMAGE) /bin/sh