# 7zplus

[English](README.md) · [下载](https://github.com/tzhaos/7zp-rust/releases/latest)

<img src="assets/brand/logo.svg" alt="7zplus" width="80" />

使用 Rust 与 GPUI Kit 构建的原生 Windows 压缩包管理器，内置 7-Zip 26.03 命令行引擎。

## 下载

前往 [GitHub Releases](https://github.com/tzhaos/7zp-rust/releases/latest) 获取最新版本。

| 文件 | 用途 |
| --- | --- |
| `7zplus-amd64-installer.exe` | 当前用户安装，提供资源管理器集成 |
| `7zplus-amd64-portable.zip` | 完整解压后运行 `7zplus.exe` |
| `SHA256SUMS.txt` | 两个发行包的 SHA-256 校验值 |

两种发行包均包含引擎和所需 Visual C++ 运行库。绿色版表示免安装，配置与历史仍保存在用户设置目录中。

## 功能

- 浏览文件夹和压缩包，支持搜索、排序、多选和历史记录。
- 创建 7z、ZIP、TAR、gzip、bzip2、xz、WIM，选项随格式能力变化。
- 解压全部或选中条目，处理同名文件，检查完整性并取消任务。
- 在格式支持时添加、重命名、删除、复制出和移出压缩包条目；读写 ZIP 注释。
- 使用 Windows 关联应用打开解压后的文件。
- 用户级资源管理器集成与显式文件类型注册，不替换系统默认关联。
- 简体中文、繁体中文、英文；浅色、深色及跟随系统主题。
- 可配置字体、字号和 33 个命令的快捷键。

操作调用内置 `7z.exe`，不依赖官方文件管理器界面。外部编辑器对解压后文件的修改不会自动回写压缩包。

## 菜单与快捷键

| 菜单 | 功能 |
| --- | --- |
| 文件 | 打开、另存为、压缩包历史、清空历史、退出 |
| 操作 | 解压到、添加、压缩包属性、注释、测试 |
| 工具 | 应用、系统、高级、关于 |

排序与选择位于文件表头，导航与刷新位于路径栏。新建压缩包保留在主页和文件夹右键菜单中。

在 **应用 → 键盘快捷键** 中搜索命令、录入或取消绑定、处理冲突，并恢复单项或全部默认值。菜单标注随保存结果同步更新。

| 默认快捷键 | 操作 |
| --- | --- |
| Ctrl+O / Ctrl+N / Ctrl+Shift+S | 打开 / 新建 / 另存压缩包 |
| Alt+E / Alt+A / Alt+T | 解压到 / 添加 / 测试 |
| Alt+I / Alt+M | 压缩包属性 / 注释 |
| F5 / F2 / Delete | 刷新 / 重命名 / 删除 |
| Ctrl+Shift+C / Ctrl+Shift+M | 复制到 / 移动到 |
| Ctrl+L / Ctrl+F | 路径栏 / 搜索 |
| Ctrl+A / Ctrl+I | 全选 / 反选 |
| Alt+Left / Alt+Up | 返回 / 上一级 |
| Ctrl+, | 应用设置 |
| Ctrl+Alt+S / Ctrl+Alt+A / Ctrl+Alt+I | 系统 / 高级 / 关于 |

Enter、Shift+Enter、Ctrl+Page Down、方向键、空格、Backspace、Escape 和 Shift+F10 保留固定导航行为。文本编辑快捷键保持输入框自身的作用范围。

## 构建

需要 Windows x64、Visual Studio C++ Build Tools 与 Windows SDK、PowerShell，以及 `rust-toolchain.toml` 固定的 Rust 工具链（当前为 1.95.0）。首次下载依赖需要网络。

```powershell
./tools/build.ps1 -ReleaseRepository tzhaos/7zp-rust -Package All
./bin/7zplus.exe
```

`-Package Run` 构建直接运行目录；`Portable` 额外生成 ZIP，`Installer` 额外生成安装包，`All` 生成两种包。产物位于 `bin/` 和 `dist/`，请保留主程序旁的 `runtime/7zip/`。

## 更新与发布

关于页检查 GitHub 最新稳定版本，启动时检查可选。点击**下载并安装更新**后，程序根据当前副本是否登记为安装版，自动选择安装包或绿色版 ZIP。下载可取消；程序依据发行包的 SHA-256 清单校验，准备好后退出，由独立更新助手完成安装并重启。助手会备份被替换的文件，失败时恢复；更新中断后，下次启动会尝试恢复。绿色版目录需要可写权限。未配置发布仓库的构建仍可运行，但无法检查更新。

通过 `./tools/version.ps1 -Part Patch`（或 `Minor` / `Major`）推进工作区版本并同步 Cargo.lock。提交两个文件后，创建并推送对应的 `vX.Y.Z` 标签。

Windows 工作流构建分支和 Pull Request。匹配版本号的标签在验证后发布，使用已有匹配标签手动触发时创建草稿。内部文件 `dist/build-info.json` 记录版本、提交、仓库、目标平台、工作区状态和校验来源。发布前回下载上传的包进行校验；重复运行保留已公开的产物。

## 设置与诊断

设置与历史位于 `%LOCALAPPDATA%\7zplus-rust`，采用原子写入。缺失配置时使用默认值；无法读取或格式错误时明确报错。字体根据已安装字体校验，不会静默替换缺失的字体名称。

`logs/` 子目录中的日志按 UTC 日期轮转，最多保留 14 个文件，记录操作失败与错误链；应用不记录命令参数和密码。

## 架构

| Crate | 职责 |
| --- | --- |
| `cardo-7zp-app` | 启动与打包 |
| `cardo-7zp-ui` | 工作区、设置、弹窗和命令分发 |
| `cardo-7zp-core` | 配置、本地化、历史与快捷键定义 |
| `cardo-7zp-engine` | 7-Zip 适配、操作与进度 |
| `cardo-7zp-requests` | 后台请求、发布检查与更新下载 |
| `cardo-7zp-platform` | Windows 集成、注册策略、更新应用与单实例 |
| `cardo-7zp-explorer` | 独立于 GPUI 和引擎适配器的资源管理器扩展 |
| `cardo-7zp-commands` | 共用命令标识、路由和请求数据 |
| `cardo-ui` | 可复用设置、菜单、提示、文本、字体与主题组件 |
| `cardo-runtime` | 原子存储、Fluent 语言目录、诊断和注册表归属检查 |

业务操作不在 render 中执行。产品模块负责值与持久化，共享组件负责展示。弹窗共用框架和大图标提示。命令菜单采用 Windows 原生表面，设置选择器沿用自绘主题。受限文本使用省略号，仅在实际截断时提供全文提示。

## 范围与验证

目标是官方 7-Zip 26.03 File Manager 的用户功能，目前仍处于早期阶段，尚未完整覆盖。

| 范围 | 限制 |
| --- | --- |
| 文件与编辑 | 编辑主要针对压缩包条目，完整文件系统管理和集成查看器、编辑器仍未完成。 |
| 查看与收藏 | 嵌套压缩包导航、扁平与图标视图、双栏、收藏仍未完成。 |
| 工具与选项 | 尚未实现基准测试和全部上游选项。 |
| 帮助 | 关于页包含版本和更新，没有集成帮助页。 |
| Windows | 已有集成，尚未全面验证各系统版本、归属冲突和安装中断场景。 |

已验证本地构建与部分引擎操作，包括 v0.2.1 文件清单修复及 7z/ZIP 创建、更新、删除、校验文件生成。这不代表完整验证了 GUI、安装升级、取消操作、无障碍、DPI 或多显示器。部分原生窗口生命周期和无障碍诊断仍待处理。

## 第三方组件

引擎来自 [7-Zip](https://www.7-zip.org/)，请保留附带许可与声明。小图标来自 [Microsoft Fluent UI System Icons](https://github.com/microsoft/fluentui-system-icons)，采用附带的 [MIT 许可](assets/fluent/LICENSE)。品牌和工具栏图标为项目原创资源。Visual C++ 运行库依据 Microsoft 的再分发条款提供。
