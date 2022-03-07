Replaces Flutter's bundled Dart SDK with the macOS arm64 version

Inspired by https://github.com/Rexios80/flutter_m1_patcher, and installation of standalone dart is not required

# Getting Started

- download the executable from https://github.com/Kylin3216/flutter_m1_patcher/releases/download/0.1.0/flutter_m1_patcher
- chmod +x flutter_m1_patcher
- flutter_m1_patcher -h
```
flutter_m1_patcher -h
USAGE:
    flutter_m1_patcher [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -d, --debug          debug 信息
    -h, --help           Print help information
    -p, --path <PATH>    flutter所在目录
    -V, --version        Print version information

SUBCOMMANDS:
    help      Print this message or the help of the given subcommand(s)
    patch     执行替换dart sdk
    revert    重置为默认dart sdk
```