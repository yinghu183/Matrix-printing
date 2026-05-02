# Matrix Printing Tool — 格子纸文字排版工具

将文字排版到格子纸（作文纸/方格纸）图片上，生成可直接打印的图片。

## 版本说明

本仓库包含两个独立版本，分别在两个分支中：

| 分支 | 版本 | 技术栈 | 文件大小 | 适用场景 |
|------|------|--------|----------|----------|
| [`main`](../../tree/main) | Python 原版 | Python + tkinter + Pillow | ~150MB（含运行环境） | 开发/定制 |
| **[`rust`](../../tree/rust)** | **Rust 重写版** | Rust + egui + fontdue | **6MB（单文件）** | **日常使用（推荐）** |

### 为什么推荐 Rust 版？

- **极速启动** — 瞬启，无 Python 运行时加载
- **体积小** — 6MB 单文件 vs 150MB Python 环境
- **内存低** — 15–30MB vs 80–120MB
- **免环境** — 复制 exe 即用，无需安装任何依赖

## Rust 版使用方法

### Windows 用户

1. 从 Release 页面下载 `matrix-printing.exe`
2. 确保同目录下有 `fonts/` 文件夹，其中包含中文字体文件（.ttf 或 .otf）
3. 双击 exe 运行

### 文件夹结构

```
matrix-printing/
├── matrix-printing.exe   ← 主程序
├── fonts/                ← 字体文件夹（放 .ttf/.otf 文件）
├── uploads/              ← 上传的格子纸图片（自动创建）
├── output/               ← 生成图片输出（自动创建）
└── config/               ← 参数预设（自动创建）
```

### 操作步骤

1. **设置网格参数** — 在左侧面板输入每行格子数和格子行数
2. **上传格子纸图片** — 点击「上传格子图片」，选择格子纸照片
3. **选择字体** — 程序会自动扫描 `fonts/` 文件夹
4. **微调参数** — 拖动红色参考线对齐纸上的格子，调整 X/Y 偏移、字体大小等
5. **粘贴文字** — 在中间区域粘贴要排版的内容
6. **预览确认** — 右侧实时预览排版效果
7. **生成图片** — 点击「生成图片」，选择保存位置

### 常用调整技巧

- **红色参考线没对齐** → 调整「起始X」「起始Y」移动整个网格
- **文字不在格子正中央** → 微调「X偏移」「Y偏移」
- **参考线太细看不清** → 增大「线条粗细」参数
- **字体太小/太大** → 修改「字体大小」
- **想缩放整张图** → 在「输出尺寸」中选择预设尺寸

## 从源码构建（Rust 版）

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆并切换到 Rust 分支
git clone https://github.com/yinghu183/Matrix-printing.git
cd Matrix-printing
git checkout rust

# 确保 fonts/ 文件夹中有中文字体

# 构建（本地）
cargo build --release

# 构建（Windows 交叉编译）
cargo install cross
cross build --target x86_64-pc-windows-gnu --release
```

## 技术

- Rust + egui/eframe (GUI)
- image (图片处理)
- fontdue (字体渲染)

## Python 原版

如需查看 Python 版代码，请切换到 [`main` 分支](../../tree/main)。

## 项目来源

本项目是原 Python/tkinter 版 Matrix-printing 的 Rust 重写版：

| 对比 | Python 版 | Rust 版 |
|------|----------|---------|
| 文件大小 | ~150MB（含运行环境） | 6MB（单文件） |
| 启动速度 | 1-2 秒 | 瞬启 |
| 内存占用 | 80–120MB | 15–30MB |
| 分发 | 需装 Python | 复制 exe 即可 |
