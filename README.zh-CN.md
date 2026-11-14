# adb-qrcode

[English](README.md) | 简体中文

在终端生成 Android 无线调试配对二维码，并可通过 ADB 自动发现和配对设备。

## 准备

- Rust 和 Cargo（编译程序需要）。
- Android 11 或更高版本的设备，启用开发者选项和无线调试。
- [Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools) 中的 `adb`，并确保它在 `PATH` 中。
- 手机和电脑连接同一网络，并允许 mDNS 设备发现。

## 使用

```sh
cargo run --release -- --pair
```

在手机上打开「开发者选项 → 无线调试 → 使用二维码配对设备」，扫描终端中的二维码。程序等待设备公布配对服务后执行 `adb pair`，默认等待 120 秒；可用 `--timeout 60` 指定秒数。配对成功后，如果设备未自动连接，可在手机无线调试页面查看 IP 地址和调试端口，然后运行 `adb connect IP:端口`。调试端口可能与配对端口不同。

其他用法：

```sh
# 同时保存 PNG，并自动配对
cargo run --release -- --pair --png adb.png

# 仅生成二维码；扫码后按提示使用 adb mdns services 和 adb pair 手动配对
cargo run --release

# 输出原始内容，供其他程序使用
cargo run --quiet -- --payload-only

# 指定名称和密码
cargo run --release -- --pair --service-name studio-example --password abc123
```

服务名称支持 1–63 个 ASCII 字母、数字或连字符；密码支持 1–64 个 ASCII 字母或数字。默认名称和密码使用系统安全随机数生成。二维码、PNG、原始内容和配对密码均含配对凭据，请仅与可信设备分享。使用 `--password` 时密码也可能被命令行历史记录保存。

二维码格式及配对机制参见 [AOSP ADB Wi-Fi 文档](https://android.googlesource.com/platform/packages/modules/adb/+/HEAD/docs/dev/adb_wifi.md)。

## 构建与发布

推送和 Pull Request 会构建 Linux x86-64、Windows x86-64 及 macOS（Intel 和 Apple Silicon）版本，并将 ZIP 包上传为工作流产物。推送标签后，相同的压缩包会自动作为 GitHub Release 附件发布。例如：

```sh
git tag v0.1.0
git push origin v0.1.0
```

## 许可证

[MIT](LICENSE)
