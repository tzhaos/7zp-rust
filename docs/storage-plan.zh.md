# 配置与存储重构方案

目标版本：Plus7z 0.2.4 / Cardo 0.2.0。

## 介质边界

| 介质 | 使用范围 | 不存放 |
| --- | --- | --- |
| TOML | 低频、可手工编辑的配置：语言、主题、字体、快捷键、行为和集成选项 | 历史、任务进度、更新恢复记录 |
| JSON | GitHub API、进程命令、构建清单，以及数据库中不需检索的结构化状态载荷 | 独立的实时配置或历史文件 |
| SQLite | 需要事务的本地状态：有序历史、最近解压目录、更新准备及结果 | 密码、日志、压缩包内容、用户配置、Windows 注册表 |

每类数据只有一份权威来源。SQLite 用于事务和关系查询，不因目前只有 20 条历史宣称性能提升。日志继续使用滚动日志；临时文件由 tempfile 管理；注册表归产品平台模块。敏感凭据不写入这三种介质。

## Cardo 基础设施

- storage::Store：带路径上下文的文件读写和同目录原子替换，用于明确的文件及交换用途。
- config::Snapshot<T>：类型化 TOML 快照；跨进程写锁；保存前比较读入原文，外部修改报告冲突；使用 toml_edit 更新变化字段并保留无关字段和注释；保留上一份有效原文备份。
- database::Database：bundled SQLite；每次操作独立连接；5 秒 busy timeout、WAL、synchronous=FULL、foreign_keys=ON；IMMEDIATE 短事务；产品提供建表 SQL、application_id 和精确 schema 版本。
- 只初始化空数据库；现有数据库必须匹配归属和版本，不执行任何版本转换。损坏或不匹配时明确报错，不删除或重建。提供一致性备份与按需完整性检查。

配置快照和数据库均不依赖 GPUI。产品在启动阶段或后台线程执行 I/O。跨介质和注册表操作不宣称拥有统一事务；失败分别报告。

## p7z 接入

新数据目录为 %LOCALAPPDATA%\p7z：

- settings.toml：配置唯一来源，schema_version = 1。
- state.sqlite3：运行数据唯一来源；历史使用关系表，路径唯一、有序、最多 20 条；更新结果和清除待处理计划在同一事务提交。
- logs/、updates/：诊断日志和更新临时文件。

读取顺序为默认值 → TOML 已提供字段 → 本次启动 --lang。配置保存一次提交行为、外观、语言和主题。TOML 无 null，取消快捷键使用 "disabled"；省略某个动作沿用默认绑定。手工修改配置后重启应用，运行中的旧快照不会覆盖已检测到的外部编辑。

配置缺失使用默认值，格式错误、读取失败和无效值明确报错。未知配置字段保留原文；schema_version 必须匹配。只使用新目录，不读取、导入或修改其他版本的数据，不提供兼容别名、双写或自动转换。

## 备份和恢复

TOML 保存使用同目录临时文件、同步和替换，保存前保留 settings.toml.bak。解析错误不自动回退，避免掩盖配置问题。手工恢复前退出应用。

数据库备份使用 SQLite backup API，不直接复制正在使用的数据库主文件。需要恢复时退出主程序和更新助手，保留原数据库及 WAL/SHM 文件，再恢复确认有效且版本匹配的备份。WAL 数据库应位于本机文件系统，不放入网络共享目录。备份与恢复不能代替对操作失败的报告。

## 分批实施与发布

1. Plus7z/p7z 改名；所有配套 crate 采用 p7z-*，共用基建采用 cardo-*。
2. 实现并提交 Cardo TOML/SQLite 基建，再接入 p7z 的设置、历史和更新状态。
3. 完成银色棱甲 logo、Cardo 中英文 README 和来源说明。
4. Cardo 独立构建、执行 tools/build.ps1 和检查发行包。按项目规则不增加测试或冒烟测试代码；分别报告构建结果和实际运行观察。
5. 汇总版本说明、提交各批次、推送 Cardo 提交及主仓库 v0.2.4 tag。tag 推送成功即结束，不等待远端工作流。

## 官方资料

- [TOML 规范](https://toml.io/en/v1.0.0)：可读配置与明确的数据类型。
- [JSON RFC 8259](https://www.rfc-editor.org/rfc/rfc8259)：数据交换格式。
- [SQLite 适用场景](https://www.sqlite.org/whentouse.html)：本地应用与单写者边界。
- [WAL](https://www.sqlite.org/wal.html)、[PRAGMA](https://www.sqlite.org/pragma.html)、[备份](https://www.sqlite.org/backup.html)。
- [rusqlite](https://docs.rs/rusqlite/0.40.2/rusqlite/)；toml_edit API 以 Cargo.lock 固定版本源码为准。
