替换Flutter默认DartSDK为macOS arm64版本

受https://github.com/Rexios80/flutter_m1_patcher启发，无需再单独安装dart

# 开始使用

- 从https://github.com/Kylin3216/flutter_m1_patcher/releases/download/0.1.0/flutter_m1_patcher下载最新版本
- 授予权限```chmod +x flutter_m1_patcher```
- 执行```flutter_m1_patcher -h```获取帮助信息

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