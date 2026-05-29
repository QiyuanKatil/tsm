# TSM — Tiny Source Manager

🇨🇳 **专为中国大陆开发者打造的一键换源工具**

> **本项目基于 [cmirror](https://github.com/ox01024/cmirror) 二次开发。**

TSM 是一个基于 Rust 编写的跨平台命令行工具，旨在解决国内开发环境依赖下载速度慢、
配置繁琐的问题。它提供"并发测速-对比-自动配置"的一站式解决方案，支持 pip, npm,
docker 等多种常见开发工具。

### 🆚 相比 cmirror 的增强

- **首次安装默认配置** — 初次运行自动生成用户配置文件（内置镜像源），后续升级不
  覆盖，用户配置为唯一数据源
- **交互式工具开关** (`tsm tools`) — 可视化多选启用/禁用工具，禁用工具的命令自动
  拦截
- **自定义源管理** (`tsm config add/rm`) — 添加/删除自定义镜像源，交互式多选删除
- **项目更名** — `tsm`（Tiny Source Manager），更短更好记
- **命令重命名** — `status` → `ls`，更直观

## ✨ 核心功能

- **⚡️ 极速体验**: 使用 HTTP/HTTPS `HEAD` 请求并发测试所有镜像源延迟，精准计算
  TTFB (Time To First Byte)。
- **🛡️ 安全无忧**: 修改任何配置前强制自动备份，支持一键恢复 (`restore`)。
- **🧠 智能推荐**: 支持 `--fastest` 参数，自动选择并应用当前网络环境下最快的源。
- **📊 状态透视**: `tsm ls` 一目了然地查看当前所有工具正在使用的源地址及状态。
- **➕ 自定义源**: `tsm config add` 添加自定义镜像源，配置即所得。
- **🔘 工具开关**: `tsm tools` 交互式管理工具启用/禁用，禁用工具的命令会提示
  `not enabled`。

## 📦 支持列表

| 工具                    | 配置文件路径                   | 备注                      |
| :---------------------- | :----------------------------- | :------------------------ |
| **pip** (Python)        | `~/.pip/pip.conf` (Linux/Mac)  | 支持 venv 及全局配置      |
| **uv** (Python)         | `uv.toml`                      | 优先项目级配置，其次全局  |
| **conda** (Python)      | `~/.condarc`                   | 自动配置 channels         |
| **npm** (Node.js)       | `~/.npmrc`                     | --                        |
| **brew** (Homebrew)     | 环境变量 `HOMEBREW_API_DOMAIN` | 输出配置指引              |
| **cargo** (Rust)        | `~/.cargo/config.toml`         | 支持 sparse 协议          |
| **go** (Golang)         | 通过 `go env -w` 设置          | GOPROXY                   |
| **docker**              | `/etc/docker/daemon.json`      | 需要 sudo                 |
| **apt** (Debian/Ubuntu) | `/etc/apt/sources.list`        | 自动检测发行版，需要 sudo |

## 🚀 快速开始

### 方法一: 从 Release 下载

```bash
# Linux x64
wget https://github.com/QiyuanKatil/tsm/releases/latest/download/tsm-linux-x64.tar.gz
tar -xzf tsm-linux-x64.tar.gz
chmod +x tsm
sudo mv tsm /usr/local/bin/
tsm --help
```

```bash
# macOS ARM64
curl -L -o tsm-macos-arm64.tar.gz https://github.com/QiyuanKatil/tsm/releases/latest/download/tsm-macos-arm64.tar.gz
tar -xzf tsm-macos-arm64.tar.gz
chmod +x tsm
sudo mv tsm /usr/local/bin/
tsm --help
```

### 方法二: 源码编译

```bash
git clone https://github.com/QiyuanKatil/tsm.git
cd tsm
cargo build --release
./target/release/tsm --help
```

## 🎯 使用示例

### 查看当前所有工具的源状态

```bash
$ tsm ls
```

### 测试单一工具的所有镜像源

```bash
$ tsm test pip
```

### 一键应用最快源

```bash
$ tsm use pip --fastest
```

### 指定源

```bash
$ tsm use pip aliyun
```

### 添加自定义源

```bash
$ tsm config add pip https://example.com/pypi/simple/
```

### 删除自定义源（交互式多选）

```bash
$ tsm config rm pip
```

### 管理工具启用/禁用

```bash
$ tsm tools
# 空格切换 ✓，回车保存
```

### 恢复配置

```bash
$ tsm restore pip
```

## 🗺️ 命令总览

| 命令                                 | 功能                       |
| :----------------------------------- | :------------------------- |
| `tsm ls [tool]`                      | 列出当前镜像配置           |
| `tsm test <tool>`                    | 基准测试所有源             |
| `tsm use <tool> <name> \| --fastest` | 应用新镜像                 |
| `tsm restore <tool>`                 | 恢复到备份或默认           |
| `tsm config add <tool> <url>`        | 添加自定义镜像源           |
| `tsm config rm <tool>`               | 删除自定义镜像源（交互式） |
| `tsm tools`                          | 启用/禁用工具（交互式）    |

## 🗺️ 配置目录

```
~/.config/tsm/
├── mirrors.json    # 用户镜像源配置（首次运行自动从内置源初始化）
└── settings.toml   # 启用/禁用的工具列表
```

## 📜 开源协议

MIT
