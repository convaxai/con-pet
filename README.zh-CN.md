# Con Pet

[English（默认）](README.md) | [简体中文](README.zh-CN.md)

Con Pet 是一个 Windows / macOS 桌面常驻应用：当用户在任意应用中连续键入配置的关键词时，在屏幕上播放一段透明、鼠标穿透的宠物动画。

## 当前能力

- 只监听刚刚键入的按键，默认关键词为 `codex`，英文匹配忽略大小写。
- 不读取输入框已有内容，不做 OCR，不读取剪贴板，也不把按键或匹配缓冲写入磁盘。
- 匹配缓冲最多保留一个关键词的长度；退格会撤回一个字符，快捷键、空格、回车等会中断序列，超时后的旧输入不能参与匹配。
- Windows 和 macOS 稳定支持物理键盘英文 A–Z。若系统 Hook 直接提供 Unicode 提交文本，也可以匹配中文；主流中文输入法通常只向全局 Hook 暴露拼音对应的物理按键，因此不保证匹配最终上屏汉字。
- 随包内置 100 个热门 Codex Pets，离线即可使用，同时过滤 Leaders 和已知政治人物内容；也会自动发现 `~/.codex/pets/*/pet.json`。
- 独立 Pet 图鉴支持搜索、悬停动画预览，以及标准 Pet ZIP 直接导入。
- 每次触发会在 Codex Pet 的 9 个标准动作中随机选择，并避免连续重复同一个动作。
- 内置更大的“蜘蛛侠 4 · 吊线摆荡”动画，从屏幕顶部居中位置拉丝摆荡。
- 支持 0.4–2.5 倍缩放、循环次数、多屏选择、四角、顶部居中、随机位置和固定坐标。
- 动画窗口透明、置顶、鼠标穿透，不会抢夺当前输入焦点。
- 设置窗关闭后继续驻留系统托盘，并支持配置开机自启。
- 界面支持独立的简体中文和英文切换，不会在同一页面混排两种语言。

## 路线图 / TODO

- [ ] 增加仅保存在本地的键入活动统计，例如聚合按键次数和每日趋势。只缓存计数，不保存键入内容、具体按键或按键序列，也不上传任何统计数据。
- [ ] 探索常驻 Pet 的互动养成，包括抚摸、喂食、玩耍、心情和陪伴等级等机制。
- [ ] 增加挂机成长机制，让 Pet 在陪伴用户的过程中随时间成长、积累经验，并解锁新的状态或动画。

## Pet 包格式

Con Pet 直接兼容 Codex Pet 标准包：

```text
my-pet/
├── pet.json
└── spritesheet.webp
```

`pet.json` 示例：

```json
{
  "id": "my-pet",
  "displayName": "My Pet",
  "description": "A tiny companion.",
  "spritesheetPath": "spritesheet.webp"
}
```

标准图集为 `1536×1872`，共 8 列 × 9 行，每格 `192×208`。动作行依次为：待机、向右跑、向左跑、挥手、跳跃、失败、等待、小跑、检查。也兼容 Codex Pets 当前使用的 8×10 / 8×11 扩展图集，不会纵向压缩。“蜘蛛侠 4 · 吊线摆荡”使用前三行连续存放 20 帧。

## 本地开发

依赖：Node.js 20+、pnpm 10+、Rust stable，以及 Tauri 2 对应平台的系统依赖。

```bash
pnpm install
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri dev
```

构建安装包：

```bash
pnpm tauri build
```

macOS 首次运行需要在“系统设置 → 隐私与安全性 → 输入监控”中允许 Con Pet，然后重启应用。应用不需要屏幕录制权限。

macOS 的输入监控授权与代码签名的 designated requirement 绑定。ad-hoc 开发构建会把 requirement 绑定到当前 CDHash，因此普通退出或重启不会丢失授权，但每次重新编译并替换 `.app` 后都需要重新授权。稳定发布必须使用同一个 Apple Development 或 Developer ID Application 证书签名；可先用 `security find-identity -v -p codesigning` 检查本机身份。这个限制来自 macOS TCC，不是 Con Pet 主动重置权限。

当前随包 100 个 Pet 资源约 170 MiB，arm64 macOS `.app` 实测约 186 MiB。资源直接从应用包读取，首次启动不会再复制到用户目录。

## 实现选型

- [Tauri 2](https://github.com/tauri-apps/tauri)：跨平台窗口、托盘、事件和打包。
- [keytap](https://github.com/jamiepine/keytap)：只读的全局键盘事件监听。
- [tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions)：macOS 输入监控权限检查与设置引导。
- React、Tailwind CSS 和 Motion：设置页面渲染、设计样式和组件动画。

详细依赖与许可说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## 特别鸣谢

Con Pet 的实现离不开以下开源项目分享的代码、经验与设计思路：

- [BongoCat](https://github.com/ayangweb/BongoCat)：跨平台 Tauri 桌宠架构、后台生命周期和 macOS 权限体验的主要参考。
- [YAKC](https://github.com/iammodev/YAKC)：透明、置顶、鼠标穿透的小型桌面浮层实现参考。
- [beUI](https://github.com/starc007/ui-components)：设置页面中 Motion 组件的代码来源和设计灵感；本项目在其基础上进行了本地适配。
- [keytap](https://github.com/jamiepine/keytap)：专注、只读的跨平台键盘监听实现，让原生输入链路更小、更稳定。
- [tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions)：为 macOS 输入监控授权提供了清晰可靠的桥接能力。
- [Codex Pets](https://codex-pets.net/) 及社区创作者：提供了 Pet 包格式、动作状态约定和丰富的宠物生态，让离线图鉴成为可能。

感谢以上项目的所有维护者和贡献者。

## 验证与打包

仓库 CI 在 macOS 和 Windows 上执行前端单元测试、TypeScript/Vite 生产构建、Rust 格式检查、Clippy 和 Rust 单元测试。macOS 的输入监听与授权生命周期仍需在具有输入监控权限的真机上手动验证。

`main` 分支 CI 通过后，`Package installers` 工作流会生成 macOS Universal DMG 和 Windows x64 NSIS 安装程序，产物名称包含提交 SHA 并保留 30 天。该工作流也可以手动运行。

## License

代码使用 MIT License。内置或用户选择的 Codex Pet 图像仍归各自资源权利人所有。
