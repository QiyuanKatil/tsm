# PRD: tsm (Tiny Source Manager)

| 项目         | 内容                                                                     |
| :----------- | :----------------------------------------------------------------------- |
| **产品名称** | tsm                                                                      |
| **核心价值** | 一键解决中国大陆开发环境网络慢的问题，提供"测速-对比-自动配置"闭环方案。 |
| **技术栈**   | Rust (Clap, Tokio, Reqwest)                                              |
| **目标用户** | 开发人员、运维工程师、Linux 爱好者                                       |

---

## 1. 产品概述

**tsm** 是一个跨平台的命令行工具，基于 [cmirror](https://github.com/ox01024/cmirror) 二次开发。它通过内置的高质量国内源列表（阿里云、腾讯云、清华 TUNA、中科大 USTC 等），并发测试网络延迟，并支持**一键修改**系统或语言包管理器的配置文件。它解决了手动搜索源、手动修改配置繁琐、且不知道哪个源当前最快的痛点。

## 2. 核心功能

### 2.1 并发测速 (`tsm test`)

- 对指定工具的所有镜像源同时发起 HTTP HEAD 请求
- 计算 TTFB (Time To First Byte) 延迟
- 表格展示排序结果和推荐

### 2.2 自动换源 (`tsm use`)

- 支持按名称指定镜像源：`tsm use pip aliyun`
- 支持 `--fastest` 参数自动选择最快源：`tsm use pip --fastest`
- 修改前自动备份原配置文件

### 2.3 状态查看 (`tsm ls`)

- 查看所有工具的当前配置状态
- 显示当前源 URL 和匹配的镜像名称

### 2.4 自定义源管理 (`tsm config`)

- `tsm config add <tool> <url>` — 添加自定义镜像源
- `tsm config rm <tool>` — 交互式删除自定义源
- 用户配置为唯一数据源，初次运行自动从内置源初始化

### 2.5 工具启用/禁用 (`tsm tools`)

- 交互式多选切换工具启用状态
- 新安装仅启用 `docker`、`npm`
- 禁用工具执行相关命令时提示 `{tool} not enabled`

### 2.6 配置恢复 (`tsm restore`)

- 自动备份配置文件（`{file}.bak.{timestamp}`）
- 恢复到最近一次备份

## 3. CLI 命令总览

```
tsm ls [tool]            列出当前镜像配置
tsm test <tool>          基准测试所有镜像源
tsm use <tool> [source]  应用指定源或 --fastest
tsm restore <tool>       恢复到备份
tsm config add <tool> <url>  添加自定义源
tsm config rm <tool>     删除自定义源（交互式）
tsm tools                启用/禁用工具（交互式）
```

## 4. 支持的工具

| 工具 | 配置方式 | 需要 sudo |
| :--- | :------- | :-------- |
| pip | `~/.pip/pip.conf` | 否 |
| npm | `~/.npmrc` | 否 |
| docker | `/etc/docker/daemon.json` | 是 |
| go | `go env -w GOPROXY` | 否 |
| cargo | `~/.cargo/config.toml` | 否 |
| brew | 环境变量 | 否 |
| apt | `/etc/apt/sources.list` | 是 |
| uv | `~/.config/uv/uv.toml` | 否 |
| conda | `~/.condarc` | 否 |

## 5. 技术架构

### 5.1 模块结构

```
src/
├── main.rs        CLI 入口、命令解析、handler 分发
├── config.rs      镜像源读取/合并 + 用户设置管理
├── sources/       各工具 SourceManager 实现
│   ├── mod.rs     get_manager 分发 + SUPPORTED_TOOLS 常量
│   ├── pip.rs
│   ├── npm.rs
│   ├── docker.rs
│   └── ...
├── traits.rs      SourceManager trait 定义
├── types.rs       Mirror / BenchmarkResult 类型
├── utils.rs       测速、备份、URL 工具函数
└── error.rs       自定义错误类型
```

### 5.2 关键架构决策

- **配置初始化**：首次运行自动将内置 JSON 复制到 `~/.config/tsm/mirrors.json`，后续直接读取用户配置，无合并开销
- **设置存储**：`~/.config/tsm/settings.toml`，字段 `enabled_tools`
- **默认启用**：首次运行仅 `docker`、`npm` 启用

## 6. 用户配置目录

```
~/.config/tsm/
├── mirrors.json      用户镜像源配置（首次运行自动初始化）
└── settings.toml     用户通过 tools 管理的启用/禁用列表
```

## 7. 风险与注意事项

- 需要 root 权限的工具（docker、apt）在执行时会有提示
- 自动检测发行版（apt 区分 Ubuntu/Debian）
- 配置文件修改前自动备份
