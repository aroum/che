# dual-yazi 项目指引

## 相关仓库

- 主仓库: https://github.com/jtianling/dual-yazi
- Homebrew tap 仓库: https://github.com/jtianling/homebrew-tap
- 本地 tap 工作目录: `/Users/jtianling/workspace/homebrew-tap`

## Tag 命名规则

- 格式: `vX.Y.Z-dual` (例: `v0.1.0-dual`)
- `-dual` 后缀用于和上游 yazi 的 `vX.Y.Z` 区分, 避免版本号撞车

## 版本发布流程

每次发布新版本到 brew tap, 按顺序执行下面 4 步.

### 1. 在主仓库打 tag 并推送

```sh
cd /Users/jtianling/workspace/dual-yazi
git tag -a vX.Y.Z-dual -m "Release vX.Y.Z-dual"
git push origin vX.Y.Z-dual
```

### 2. 计算 source tarball 的 sha256

```sh
curl -fsSL https://github.com/jtianling/dual-yazi/archive/refs/tags/vX.Y.Z-dual.tar.gz \
  | shasum -a 256
```

### 3. 更新 tap 仓库的 formula

修改 `homebrew-tap/Formula/dual-yazi.rb` 中的两个字段:

- `url`: 把版本号换成新 tag
- `sha256`: 替换成上一步算出的值

提交并推送:

```sh
cd /Users/jtianling/workspace/homebrew-tap
git commit -am "dual-yazi vX.Y.Z-dual"
git push
```

### 4. 验证

```sh
brew update
brew info jtianling/tap/dual-yazi   # 确认 stable 字段已更新
```

用户侧升级方式: `brew upgrade dual-yazi`.

## Formula 关键约束 (维护时注意)

- binary 名仍为 `yazi` 和 `ya`, 与上游 yazi 冲突, formula 中已声明 `conflicts_with "yazi"`
- 必须设置 `ENV["VERGEN_GIT_SHA"]` 和 `ENV["YAZI_GEN_COMPLETIONS"]`, 否则编译失败或 completions 缺失
- completions 路径: `yazi-boot/completions/`, `yazi-cli/completions/`
- 不引入预编译 bottle, 用户首次安装会本地编译 (5–10 分钟)
