# Android Gradle 配置
# 在项目初始化完成后，编辑 src-tauri/gen/android/gradle.properties

# 签名配置（生产使用）
org.gradle.jvmargs=-Xmx4096m

# 构建配置
android.useAndroidX=true
android.enableJetifier=true

# NDK 版本
android.ndkVersion=26.1.10909125

# 目标 SDK
android.compileSdk=34
