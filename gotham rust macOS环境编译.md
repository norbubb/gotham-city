#  macOS环境编译[gotham-city](https://github.com/ZenGo-X/gotham-city)

iOS静态库编译遇到的问题:


1. libgmp，因为macOS下只有x86_64的版本，所以只能自己手动编译出复合库文件(x86_64 / arm64)，然后放到/usr/local/lib/目录下;
2. 现有项目拉取,容易拉去github失败，故此需要多试几次;
3. iOS下的变异编译命令是cargo lipo --release;
4. cargo和rust的版本最好升级到最新版本;


Android:

1. 任意处创建一个名为NDK的目录（名字随意，也可不叫NDK），然后运行NDK工具包中的py脚本以编译NDK开发环境

```
cd ~
mkdir NDK
# 不同架构参数不同，按需配置即可，比如我就只需要arm64
# api参数最好按你的应用targetSDK的版本号来，比如我这里是30
# Python版本我这里是3.8，如果你是Python 2.x的话，不确定能否运行成功
python ${HOME}/Library/Android/sdk/ndk-bundle/build/tools/make_standalone_toolchain.py --api 30 --arch arm64 --install-dir NDK/arm64
python ${HOME}/Library/Android/sdk/ndk-bundle/build/tools/make_standalone_toolchain.py --api 30 --arch arm --install-dir NDK/arm
python ${HOME}/Library/Android/sdk/ndk-bundle/build/tools/make_standalone_toolchain.py --api 30 --arch x86 --install-dir NDK/x86

```

2 .  创建一个新文件～/.cargon/config.toml。该文件将告诉cargo在交叉编译期间在哪里寻找clang链接器。将以下内容添加到文件中：

```
# 同样是按需配置，如果你不需要编译其他架构，就不添加
# 相关路径最好写绝对路径，此处若用${HOME}不生效
[target.aarch64-linux-android]
ar = "/Users/你的用户名/NDK/arm64/bin/aarch64-linux-android-ar"
linker = "/Users/你的用户名/NDK/arm64/bin/aarch64-linux-android-clang"

[target.armv7-linux-androideabi]
ar = "/Users/你的用户名/NDK/arm/bin/arm-linux-androideabi-ar"
linker = "/Users/你的用户名/NDK/arm/bin/arm-linux-androideabi-clang"

[target.i686-linux-android]
ar = "/Users/你的用户名/NDK/x86/bin/i686-linux-android-ar"
linker = "/Users/你的用户名/NDK/x86/bin/i686-linux-android-clang"

```

3 . 在2中，有可能编译失败，出现未安装"aarch64-linux-android-clang" 的错误或者其他错误，错误日志如下:

```
error: failed to run custom build command for `openssl-sys v0.9.93`
note: To improve backtraces for build dependencies, set the CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG=true environment variable to enable debug information generation.

Caused by:
  process didn't exit successfully: `xxx/target/release/build/openssl-sys-a0be74610eee596c/build-script-main` (exit status: 101)
```

以上出错日志，很容易陷入是自己的openssl环境的问题中，其实不是，是因为aarch64-linux-android-clang路径的问题, 仅仅在config文件里设置linker有可能没有效果，所以需要把ndk的bin目录，如"/Users/你的用户名/NDK/arm/bin/" 路径，加入到环境变量中;

4 . 关于NDK版本的问题，如果用21.4.7075529  aarch64-linux-android-clang和aarch64-linux-android-ar 都能在bin目录下找到，但是后报错缺少libuwind 库，只有libgcc.a，这里我做了软链接，但是没效果。然后换成25.2.9519653，但是没有对应的-ar, 这里把所有的-ar都换成llvm-ar。

<mark> 注意修改上面提到的环境变量和 config配置文件</mark>

5 .  这里也会缺少libgmp的库， 所以需要交叉编译android环境的ligmp静态库，放到ndk中，用于编译客户端的静态库.

6. Android的编译命令如下:

```c
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target i686-linux-android --release

```

在编译aarch64 .so的时候有可能出现
```
Format: elf64-littleaarch64
Arch: aarch64
AddressSize: 64bit
LoadName: <Not found>
```
没有LoadName会导致Android测不能加载so库

所以要用如下命令

```
RUSTFLAGS="-C link-arg=-Wl,-soname,libclient_lib.so" cargo build --target aarch64-linux-android --release
```



### 引用

[Rust为Android应用编译so库](https://juejin.cn/post/7063357084976283678)

[Rust库交叉编译以及在Android与iOS中使用](https://blog.csdn.net/qq_17766199/article/details/128749313)

[mac linux编译安卓,Mac下交叉编译android端静态库(.a)](https://blog.csdn.net/weixin_35906794/article/details/116682502)

[Android mac 交叉编译与ffmpeg编译踩坑记 (v7a 与 v8a and 动态库与静态库)](https://blog.csdn.net/weixin_44819566/article/details/131659722)

[Can't compile on android targets #1402](https://github.com/sfackler/rust-openssl/issues/1402)

[A prebuilt GMP module for Android](https://github.com/Rupan/gmp)

[Can't locate installed Openssl development headers. #1543](https://github.com/sfackler/rust-openssl/issues/1543)

[Rust 移动端开发体验](https://juejin.cn/post/7119467751302758407)