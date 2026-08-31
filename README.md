# Con Pet

Con Pet 是一个 Windows / macOS 桌面常驻应用：当用户在任意应用中连续键入配置的关键词时，在屏幕上播放一段宠物序列帧动画。

## 当前能力

- 全局监听最近键入的字符，默认关键字为 `codex`，忽略英文大小写。
- 不读取输入框已有内容，不做 OCR，不读取剪贴板，不把按键或匹配缓冲写入磁盘。
- 匹配缓冲最多保留关键字长度；退格会撤回一个字符，快捷键、空格、回车等会中断序列，超时后旧序列不能参与匹配。
- 英文 A–Z 在 Windows 和 macOS 上稳定支持。若系统事件 Hook 直接提供 Unicode 提交文本，也会匹配中文；主流中文输入法通常只向 Hook 暴露拼音键，因此不保证最终上屏汉字。
- 随包内置 Codex Pets Liked Top 100（过滤 Leaders），离线可选；同时自动发现 `~/.codex/pets/*/pet.json`。
- 独立 Pet 图鉴提供本地预览、在线排行、搜索、单个安装、批量安装和标准 ZIP 直接导入。
- 支持 9 个标准动作和“蜘蛛侠 4 · 吊线摆荡”、动画循环次数、0.4–2.5 倍缩放。
- 内置近期《蜘蛛侠：崭新之日》限时动态表情的吊线摆荡款；选中后自动使用 1.6 倍大小、屏幕顶部居中、摆荡一次。
- 支持鼠标所在屏、主屏或随机屏幕；位置可选四角、顶部居中、随机或固定坐标。
- 动画窗口透明、置顶、鼠标穿透，不抢夺当前输入焦点；设置窗关闭后应用继续驻留托盘。

## 序列帧格式

Con Pet 直接兼容 Codex Pet 的标准包：

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

macOS 首次运行需要在“系统设置 → 隐私与安全性 → 输入监控”中允许 Con Pet，然后重启应用。应用不需要屏录制权限。

macOS 的输入监控授权与代码签名的 designated requirement 绑定。ad-hoc 开发构建会把 requirement 绑定到当前 CDHash，因此普通退出/重启不会丢失授权，但每次重新编译并替换 `.app` 后都需要重新授权。稳定发布必须使用同一个 Apple Development 或 Developer ID Application 证书签名；可先用 `security find-identity -v -p codesigning` 检查本机身份。这个限制来自 macOS TCC，不是全局监听实现主动重置权限。

当前随包 Top 100 资源约 170 MiB，arm64 macOS `.app` 实测约 186 MiB。资源直接从应用包读取，首次启动不会再复制到用户目录。

### 原生输入链路自测

Debug 构建提供一个显式环境变量，用 `rdev` 在系统事件层发送当前英文关键字，验证全局 Hook → 匹配器 → 浮层的完整链路：

```bash
PC_PET_NATIVE_INPUT_SELF_TEST=1 pnpm tauri dev
```

该入口由 `debug_assertions` 保护，不会进入 Release 行为；设置页只显示匿名的命中和渲染计数，不记录字符内容。

## 实现选型

- [Tauri 2](https://github.com/tauri-apps/tauri)：跨平台窗口、托盘和打包。
- [rdev](https://github.com/kunkunsh/rdev)：全局键盘事件；固定到 BongoCat 使用的提交版本。
- [tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions)：macOS 输入监控权限检查与引导。
- 透明小窗和监听方式参考了 [BongoCat](https://github.com/ayangweb/BongoCat) 与 [YAKC](https://github.com/iammodev/YAKC) 的成熟实践。

详细第三方说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## 验证

仓库 CI 在 macOS 和 Windows 上执行：前端单元测试、TypeScript/Vite 生产构建、Rust 单元测试。macOS 还需在有输入监控权限的真机上执行上述原生链路自测。

手动运行 GitHub Actions 的 `Package installers` 工作流，可分别产出 macOS DMG 和 Windows NSIS 安装程序。

## License

代码使用 MIT License。内置或用户选择的 Codex Pet 图像仍归各自资源权利人所有。
