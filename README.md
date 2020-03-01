# mikack-ffi

mikack 库的通用外部接口。

## 基本介绍

本项目是对 [mikack](https://github.com/Hentioe/mikack) 的 C ABI 接口导出，目的是作为多种编程语言/操作系统的开发基石。如果您需要将 mikack 用于自己的技术中，本库应该是你需要的。

## 对本库的绑定

- [mikack-dart](https://github.com/Hentioe/mikack-dart)（使用于 Dart 和 Flutter 应用）

## 编译项目

如果您不需要为 Android 等嵌入式平台构建库文件，可以略过整个编译章节，直接执行 `make` 即可达到目的。如果您需要其它平台例如 Android/IOS 或 OpenWrt 请继续往下看。

### 前提

构建本项目需要以下前提：

- Rust Stable（必须）
- Android NDK（可选，将 Android 作为目标时需要）
- OpenWrt SDK（可选，将 OpenWrt 作为目标时需要）

其中 NDK 无需添加到 PATH 中，并且我个人也不推荐添加到 PATH。

#### Android 平台

##### 创建配置文件

添加目标（Rust）: 

```bash
rustup target add x86_64-linux-android aarch64-linux-android
```

在项目根创建 `.ndk` 文件，写入以下内容:

```env
BIN_PATH=/software/android-sdk/ndk/21.0.6113669/toolchains/llvm/prebuilt/linux-x86_64/bin
```

以上只是一个例子，`BIN_PATH` 变量的值请自行替换为自己的 NDK 工具链路径，要直达 bin 目录。

##### 生成 Cargo 配置

确认创建以上配置以后，将基于存在的配置生成 Rust 工具链需要的更复杂的配置。

终端执行 `make config` 并确认是否生成 `.cargo/config` 文件，最好自行排查一下其中的路径是否有误。

##### 构建依赖

执行 `make android` 命令，完成后将生成 `dist/android` 目录。直接复制 android 目录，向 Flutter 项目的根目录覆盖粘贴即可。

## 文档

_文档还没开始写呢……_