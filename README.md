# adb-qrcode

English | [简体中文](README.zh-CN.md)

Generate Android wireless debugging pairing QR codes in the terminal and optionally discover and pair devices automatically using ADB.

## Requirements

- Rust and Cargo to build the program.
- An Android 11 or later device with Developer options and Wireless debugging enabled.
- `adb` from [Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools), available in your `PATH`.
- The phone and computer must be on the same network, with mDNS device discovery allowed.

## Usage

```sh
cargo run --release -- --pair
```

On your phone, open **Developer options → Wireless debugging → Pair device with QR code** and scan the QR code in the terminal. The program waits for the device to advertise its pairing service, then runs `adb pair`. The default timeout is 120 seconds; use `--timeout 60` to specify a timeout in seconds. If the device does not connect automatically after pairing, find its IP address and debugging port on the Wireless debugging screen and run `adb connect IP:port`. The debugging port may differ from the pairing port.

Other examples:

```sh
# Restart the adb service (runs adb kill-server followed by adb start-server)
cargo run --release -- restart-adb

# Save a PNG and pair automatically
cargo run --release -- --pair --png adb.png

# Generate a QR code only; after scanning, follow the prompts to pair manually
# using adb mdns services and adb pair
cargo run --release

# Print the raw payload for use by other programs
cargo run --quiet -- --payload-only

# Specify a service name and password
cargo run --release -- --pair --service-name studio-example --password abc123
```

The service name must contain 1–63 ASCII letters, digits, or hyphens; the password must contain 1–64 ASCII letters or digits. Default names and passwords are generated using the system's secure random number generator. QR codes, PNGs, raw payloads, and pairing passwords contain pairing credentials; share them only with trusted devices. When using `--password`, the password may also be saved in your shell history.

For details on the QR code format and pairing mechanism, see the [AOSP ADB Wi-Fi documentation](https://android.googlesource.com/platform/packages/modules/adb/+/HEAD/docs/dev/adb_wifi.md).

## Builds and releases

Pushes and pull requests build release binaries for Linux x86-64, Windows x86-64, and macOS (Intel and Apple Silicon). Download the ZIP files from the workflow run's artifacts. Pushing a tag builds the same packages and publishes them as assets of a GitHub Release. For example:

```sh
git tag v0.1.0
git push origin v0.1.0
```

## License

[MIT](LICENSE)
