# 7zplus

[English](README.md)

7zplus 是使用 Rust 编写的 Windows 原生压缩管理工具，界面基于 GPUI Kit 0.6.1，内置 7-Zip 26.03 引擎。浅色主题保留背景与内嵌白色内容面板，搭配大尺寸彩色工具栏图标。

## 功能

- 浏览压缩包和本地文件夹，搜索条目、排序、多选，以及访问最近位置。
- 创建 7z、ZIP、TAR、gzip、bzip2、xz 和 WIM 压缩包，按格式提供压缩、密码和分卷选项。
- 解压全部或选中条目，设置重名处理策略，检查完整性，以及取消后台任务。
- 对支持编辑的格式添加、重命名、删除条目，将条目复制或移出压缩包；读写 ZIP 注释。
- 使用 Windows 关联程序打开解压后的文件，计算校验和，生成或验证校验文件。
- 通过独立 Shell 扩展接入资源管理器，提供可选的用户级文件关联和单实例命令转发。
- 支持简体中文、繁体中文、英语，以及浅色和深色主题。

压缩操作使用内置引擎执行。使用外部程序打开条目后，对文件的后续修改不会自动写回压缩包。

## 构建与运行

环境要求：

- Windows x64，Rust 1.95.0，使用 `rust-toolchain.toml` 指定的 MSVC 目标。
- Visual Studio C++ Build Tools 和 Windows SDK。
- PowerShell；首次下载依赖时需要网络连接。

```powershell
./tools/build.ps1
./bin/7zplus.exe
```

脚本构建主程序与 Shell 扩展，下载并校验固定版本的 7-Zip 和 NSIS，生成以下产物：

```text
bin/
  7zplus.exe
  7-zip-plus.dll
  7zplus-amd64-installer.exe
  SHA256SUMS.txt
  runtime/7zip/
```

运行时应将 `runtime/7zip/` 保留在主程序旁。需要资源管理器集成时使用安装包。注册仅操作应用自有的用户级条目，不替换 Windows 默认压缩文件关联。

构建时传入 `-ReleaseRepository owner/repo` 可启用 GitHub 发布检查。用户配置保存在 `%LOCALAPPDATA%\7zplus-rust`。

## 命令行

```text
7zplus.exe [--lang zh-CN|zh-TW|en-US] [路径 ...]
7zplus.exe --register [dll 路径]
7zplus.exe --unregister
7zplus.exe --prepare-install <安装目录>
```

后三项命令用于安装与维护。资源管理器动作标识及完整选择列表的数据结构定义在 `crates/shell-api` 中。

常用快捷键：`Ctrl+O` 打开压缩包，`Ctrl+L` 聚焦地址栏，`Ctrl+F` 搜索，`Ctrl+A` 全选条目，`Alt+Left` 返回，`Alt+Up` 进入上级，`Alt+E` 打开解压选项，`Alt+T` 检查完整性，`F1` 打开内置帮助。其他快捷键显示在对应菜单中。

## 架构

| 目录 | 职责 |
| --- | --- |
| `src/archive` | 压缩包模型、7-Zip 进程适配、编辑、注释、校验和与进度解析，不依赖 GPUI 或注册表。 |
| `src/application` | 后台请求与结果、文件系统读取、解压工作流和发布检查。 |
| `src/platform` | Windows 文件打开与图标、单实例通信、注册表集成、邮件和安装维护。 |
| `src/settings`、`src/i18n.rs` | 配置持久化、最近位置和 Fluent 本地化。 |
| `src/ui/mod.rs` | 工作区状态、订阅、初始化，以及输入和焦点同步。 |
| `src/ui/actions` | 任务生命周期、压缩操作与设置变更的工作区协调。 |
| `src/ui/views` | 窗口框架、菜单栏、文件列表和设置页面组合。 |
| `src/ui/dialogs` | 对话框组合、压缩包创建表单和错误呈现。 |
| `src/ui/components` | 公共按钮、输入框、提示、导航、列表、设置行、图标和菜单样式。 |
| `src/ui/theme`、`src/ui/theme.rs` | 公共尺寸、语义色板和 GPUI 主题接入。 |
| `crates/shell-api` | 共享动作枚举、格式路由和序列化选择请求。 |
| `crates/shell` | 资源管理器 COM 扩展，不依赖 GPUI 和压缩引擎。 |
| `locales`、`assets` | 产品文案和嵌入式视觉资源。 |
| `tools`、`packaging` | 固定版本引擎准备、构建脚本和 NSIS 安装包。 |

UI 命令派发后台任务并处理结果；视图组合公共组件，不在渲染中执行文件、进程、网络或注册表操作。右键菜单保留点击目标和选择快照。产品文案维护在三份 Fluent 语言文件中，以语义键标识，禁止将翻译后的文字作为状态。

## 界面规范

共享控件使用 `src/ui/components` 中的组件，公共尺寸在 `src/ui/theme/metrics.rs` 修改，颜色由 `src/ui/theme.rs` 的语义色板提供。

命令按钮和单行输入框高度为 32px，命令文字使用 GPUI Kit 的 12px `XSmall` 尺寸。图标按钮为 30px 正方形。控件圆角为 4px，主面板为 12px，无箭头提示为 10px。工具栏保留 48px SVG 图标，按钮高度固定为 80px，宽度为 66-72px。页面工具通过宽度和间距动画收缩，保留当前页面，并遵循减少动态效果设置。

背景、内嵌内容面板和彩色工具栏图标共同构成界面特征。新增视图复用固定版本的 GPUI Kit API 和公共组件，资源来源记录在 [THIRD_PARTY.md](THIRD_PARTY.md)。正常构建直接使用已提交资源，不依赖 Node.js 或 Python；重生成品牌资源时，`tools/render-brand.cjs` 使用 Sharp，`tools/generate-icon.py` 使用 Pillow。

## 范围与验证

产品目标是覆盖 7-Zip 26.03 File Manager 的用户功能。当前源码仍为部分覆盖：

| 范围 | 当前能力与缺口 |
| --- | --- |
| 文件与编辑 | 已有压缩包创建、解压、编辑、属性、注释和最近位置。完整本地文件管理、集成查看与编辑尚未完成。 |
| 查看与收藏 | 已有文件夹和压缩包导航、搜索、列表排序。完整查看命令、多种布局及收藏尚未实现。 |
| 工具与选项 | 已有完整性检查、校验和，以及常规、高级、外观和关联设置。基准测试及完整的上游选项与对话框尚未覆盖。 |
| 帮助 | 内置 7-Zip 帮助、应用信息及可选发布检查。 |
| Windows 集成 | 已实现资源管理器命令、用户级注册、安装维护与单实例转发；仍需在不同 Windows 版本中验证。 |

本次源码整理仅进行格式和静态源码检查，尚未编译或在原生界面中验证。工具栏收缩、焦点行为、显示缩放和多语言布局尤其需要运行确认。只在明确要求时构建，`bin/` 中已有产物不能作为当前源码的验证依据。

## 第三方软件

上游组件和图标来源见 [THIRD_PARTY.md](THIRD_PARTY.md)。分发时保留内置 7-Zip 许可文件和 `assets/fluent/LICENSE`。
