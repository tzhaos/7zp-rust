<p align="center">
  <img src="assets/brand/logo.svg" width="112" alt="Plus7z">
</p>
<h1 align="center">Plus7z</h1>
<p align="center">使用 Rust 与 GPUI Kit 构建的原生 Windows 压缩包管理器。</p>
<p align="center">
  <a href="https://github.com/tzhaos/7zp-rust/releases/latest">下载</a> ·
  <a href="README.md">English</a> ·
  <a href="docs/reference.zh.md">使用与开发说明</a> ·
  <a href="https://github.com/tzhaos/7zp-rust/issues">反馈问题</a>
</p>
<p align="center"><strong>Windows x64</strong> · 7-Zip 26.03 · <a href="LICENSE">MIT</a></p>

## 下载

前往 **[Releases 下载安装版或绿色版](https://github.com/tzhaos/7zp-rust/releases/latest)**。两种包均包含 7-Zip 26.03 和所需的 Visual C++ 运行库。

| 文件 | 用途 |
| --- | --- |
| `p7z-amd64-installer.exe` | 当前用户安装，提供资源管理器集成 |
| `p7z-amd64-portable.zip` | 完整解压文件夹后运行 `p7z.exe` |

发行包附带 `SHA256SUMS.txt`。两种版本均可在**关于**页检查更新，设置与历史保存在 `%LOCALAPPDATA%\p7z`。

## 功能

- **浏览**文件夹与压缩包，支持搜索、排序、多选和历史记录。
- **创建与解压** 7z、ZIP、TAR、gzip、bzip2、xz 和 WIM，选项随格式能力变化。
- **管理压缩包条目**：添加、重命名、删除、复制出或移出；支持 ZIP 注释编辑。
- **处理任务**：同名文件选择、完整性检查、取消操作与简短的解压完成通知。
- **调整界面**：三种语言、浅深主题、自定义字体和 33 个可配置快捷键。
- **接入资源管理器**：显式注册到当前用户，保留系统默认文件关联。

项目仍处于早期阶段。完整文件系统管理、集成查看器与编辑器、嵌套压缩包、更多视图、收藏和基准测试尚未完成；外部编辑不会自动回写压缩包。详见[范围与验证](docs/reference.zh.md#范围与验证)。

## 构建

Windows x64 · Visual Studio C++ Build Tools + Windows SDK · PowerShell · Rust 1.95.0

```powershell
git clone --recurse-submodules https://github.com/tzhaos/7zp-rust.git
cd 7zp-rust
./tools/build.ps1 -ReleaseRepository tzhaos/7zp-rust -Package All
./bin/p7z.exe
```

已有工作区执行 `git submodule update --init --recursive`。`-Package Run` 只构建可运行目录；`All` 还会在 `dist/` 生成安装包与绿色 ZIP。

应用包名为 `p7z`，配套 crate 使用 `p7z-*`；共用的 `cardo-runtime` 和 `cardo-ui` 来自固定提交的 [Cardo](https://github.com/tzhaos/cardo-rust) 子模块。[架构、快捷键与发布流程 →](docs/reference.zh.md)

## 许可

项目采用 [MIT](LICENSE)。随包的 7-Zip、小图标资源及 Visual C++ 运行库保留各自许可，详见[第三方声明](THIRD_PARTY.md)。
