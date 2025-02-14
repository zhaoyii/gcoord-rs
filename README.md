# gcoord-rs

Gcoord-rs(Geographic Coordinates Rust)是一个处理地理坐标系的 Rust crate，提供`wgs84/gcj02/bd09`等坐标系相互转换的功能 。Gcoord-rs 是[gcoord](https://github.com/hujiulong/gcoord)的 Rust 实现。

您可以通过 cargo 依赖它:
```toml
[dependencies]
gcoord = "0.1.0"
```

## 示例

```Rust
use gcoord::{transform, Coordinate, CoordSystem};
let from = Coordinate::new(114.304569, 30.593354);
let to = transform(from, CoordSystem::WGS84, CoordSystem::GCJ02);
```

## 支持的坐标系
目标支持以下几种坐标系相互转换：

| CRS                | 坐标格式   | 说明    |
| --------           | --------- | ----- |
| gcoord.WGS84       | [lng,lat] | WGS-84坐标系，GPS设备获取的经纬度坐标   |
| gcoord.GCJ02       | [lng,lat] | GCJ-02坐标系，google中国地图、soso地图、aliyun地图、mapabc地图和高德地图所用的经纬度坐标   |
| gcoord.BD09        | [lng,lat] | BD-09坐标系，百度地图采用的经纬度坐标    |


## 🚨 注意
在发布、展示、传播数据时，请务必遵守相关法律规定

> （禁止）未经批准，在测绘活动中擅自采用国际坐标系统
> — 中华人民共和国测绘法，40 (1)
>
> 导航电子地图在公开出版、销售、传播、展示和使用前，必须进行空间位置技术处理。
>— GB 20263―2006《导航电子地图安全处理技术基本要求》，4.1