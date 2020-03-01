include .ndk
export NDK_BIN_PATH=${BIN_PATH}
unexport BIN_PATH

CARGO_BIN ?= `which cargo`
.PHONY: config android

export X86_64_ANDROID_KEY="[target.x86_64-linux-android]"
export X86_64_ANDROID_LINKER="${NDK_BIN_PATH}/x86_64-linux-android29-clang"
export X86_64_ANDROID_AR="${NDK_BIN_PATH}/x86_64-linux-android-ar"
export AARCH64_ANDROID_KEY="[target.aarch64-linux-android]"
export AARCH64_ANDROID_LINKER="${NDK_BIN_PATH}/aarch64-linux-android29-clang"
export AARCH64_ANDROID_AR="${NDK_BIN_PATH}/aarch64-linux-android-ar"

.DEFAULT_GOAL := default

config:
	@mkdir .cargo -p
	@echo -e "${X86_64_ANDROID_KEY}\nlinker = \"${X86_64_ANDROID_LINKER}\"\nar = \"${X86_64_ANDROID_AR}\"" > .cargo/config
	@echo >> .cargo/config
	@echo -e "${AARCH64_ANDROID_KEY}\nlinker = \"${AARCH64_ANDROID_LINKER}\"\nar = \"${AARCH64_ANDROID_AR}\"" >> .cargo/config

clean:
	@cargo clean

distclean:
	@rm -r dist/

default:
	@cargo build --release

android_x86_64:
	@CC=${X86_64_ANDROID_LINKER} AR=${X86_64_ANDROID_AR} cargo build --release --target=x86_64-linux-android

android_aarch64:
	@CC=${AARCH64_ANDROID_LINKER} AR=${AARCH64_ANDROID_AR} cargo build --release --target=aarch64-linux-android

android: android_x86_64 android_aarch64
	@$(eval JNILIBS := dist/android/app/src/main/jniLibs)
	@mkdir ${JNILIBS}/x86_64 -p
	@mkdir ${JNILIBS}/arm64-v8a -p
	@cp target/x86_64-linux-android/release/libmikack_ffi.so ${JNILIBS}/x86_64/
	@cp target/aarch64-linux-android/release/libmikack_ffi.so ${JNILIBS}/arm64-v8a/

all: default android