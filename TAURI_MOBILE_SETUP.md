# Williw - Tauri Mobile 集成指南

## 进度

- ✅ 创建 src-tauri 目录结构
- ✅ 配置 tauri.conf.json
- ✅ 创建 src-tauri/Cargo.toml
- ✅ 配置 frontend WASM 编译
- ⏳ 初始化 Android 项目中...

## 项目结构

```
williw/
├── src-tauri/              # Tauri 后端
│   ├── src/
│   │   └── main.rs        # Tauri 应用入口
│   ├── Cargo.toml         # Tauri 配置
│   ├── build.rs           # 构建脚本
│   └── tauri.conf.json    # Tauri 配置文件
├── frontend/               # Leptos WASM 前端
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── ...
│   └── Cargo.toml
├── api/                    # 后端 API（不用于移动版）
├── shared/                 # 共享类型
└── Cargo.toml              # 工作区配置
```

## 后续步骤

1. ✅ 等待 Android 初始化完成
2. ⏳ 安装 Android SDK / NDK（如需要）
3. ⏳ 配置本地 gradle.properties
4. ⏳ 测试编译
5. ⏳ 生成 APK

## 命令参考

```bash
# 开发模式（需要 Android 模拟器或真机）
cargo tauri android dev

# 构建 Release APK
cargo tauri android build --release

# APK 位置
src-tauri/gen/android/app/build/outputs/apk/release/
```

## 环境要求

- Rust 1.93+ ✅
- Cargo ✅
- Tauri CLI ✅
- Android SDK (API 21+)
- Android NDK
- Java JDK 17+
