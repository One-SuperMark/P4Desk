# Colloid 矢量资产

这里是 `third_party/colloid-p4desk` 的规范化派生文件，共 42 个；许可 GPL-3.0。
输入哈希、层数与输出哈希见 `assets/colloid-icons.json`。

```sh
python3 -m venv .cache/svg-tools
.cache/svg-tools/bin/pip install -r scripts/requirements-svg.txt
.cache/svg-tools/bin/python scripts/generate-colloid-icons.py
.cache/svg-tools/bin/python scripts/generate-colloid-icons.py --check
```

编译为 `colloid_icons_generated.rs` 中的静态矢量几何，固件不解析 SVG XML，也没有内嵌图标位图。滑动时使用有界的运行时透明页面缓存，停止后继续绘制原始矢量与实时指针。
