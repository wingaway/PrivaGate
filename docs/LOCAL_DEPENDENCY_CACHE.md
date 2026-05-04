# 本地依赖和缓存位置

本项目要求依赖缓存、构建产物和运行缓存放在项目目录内，避免写入 C 盘用户目录。

## Rust / Cargo

PowerShell 开发前先执行：

```powershell
.\scripts\dev-env.ps1
```

该脚本设置：

```text
CARGO_HOME=E:\CodeHub\ProofGate\.cargo-home
RUSTUP_HOME=E:\CodeHub\ProofGate\.rustup-home
CARGO_TARGET_DIR=E:\CodeHub\ProofGate\target
```

`.cargo/config.toml` 同时把 Cargo 构建产物固定到仓库内 `target/`。

## 约束

- 不把依赖下载到 `C:\Users\<user>\.cargo`。
- 不把 Rust 工具链下载到 `C:\Users\<user>\.rustup`。
- 不把构建产物放到 C 盘临时目录。
- 运行数据库、对象存储和审计日志时，Docker volume 也应显式映射到项目目录或 E 盘专用数据目录。
