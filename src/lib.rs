// 定义坐标系类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordSystem {
    WGS84,
    GCJ02,
    BD09,
}

// 定义坐标结构体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate {
    pub lng: f64,
    pub lat: f64,
}

// 错误类型
#[derive(Debug, PartialEq)]
pub enum ConvertError {
    UnsupportedConversion,
    OutOfChina,
}

impl Coordinate {
    pub fn new(lng: f64, lat: f64) -> Self {
        Self { lng, lat }
    }
}

// 核心转换实现
pub fn transform(
    coord: Coordinate,
    from: CoordSystem,
    to: CoordSystem,
) -> Result<Coordinate, ConvertError> {
    match (from, to) {
        (CoordSystem::WGS84, CoordSystem::GCJ02) => gcj02_wgs84::wgs84_to_gcj02(coord),
        (CoordSystem::GCJ02, CoordSystem::WGS84) => gcj02_wgs84::gcj02_to_wgs84(coord),
        (CoordSystem::GCJ02, CoordSystem::BD09) => Ok(gcj02_bd09::gcj02_to_bd09(coord)),
        (CoordSystem::BD09, CoordSystem::GCJ02) => Ok(gcj02_bd09::bd09_to_gcj02(coord)),
        (CoordSystem::WGS84, CoordSystem::BD09) => Ok(gcj02_bd09::gcj02_to_bd09(
            gcj02_wgs84::wgs84_to_gcj02(coord)?,
        )),
        (CoordSystem::BD09, CoordSystem::WGS84) => Ok(gcj02_wgs84::gcj02_to_wgs84(
            gcj02_bd09::bd09_to_gcj02(coord),
        ))?,
        _ => Ok(coord), // 相同坐标系直接返回
    }
}

mod gcj02_bd09 {
    use super::Coordinate;

    const BAIDU_FACTOR: f64 = std::f64::consts::PI * 3000.0 / 180.0;
    pub fn bd09_to_gcj02(coord: Coordinate) -> Coordinate {
        let x = coord.lng - 0.0065;
        let y = coord.lat - 0.006;
        let z = (x.powi(2) + y.powi(2)).sqrt() - 0.00002 * (y * BAIDU_FACTOR).sin();
        let theta = y.atan2(x) - 0.000003 * (x * BAIDU_FACTOR).cos();

        let lng = z * theta.cos();
        let lat = z * theta.sin();

        Coordinate::new(lng, lat)
    }

    pub fn gcj02_to_bd09(coord: Coordinate) -> Coordinate {
        let x = coord.lng;
        let y = coord.lat;
        let z = (x.powi(2) + y.powi(2)).sqrt() + 0.00002 * (y * BAIDU_FACTOR).sin();
        let theta = y.atan2(x) + 0.000003 * (x * BAIDU_FACTOR).cos();

        let lng: f64 = z * theta.cos() + 0.0065;
        let lat = z * theta.sin() + 0.006;

        Coordinate::new(lng, lat)
    }
}

mod gcj02_wgs84 {
    use super::{ConvertError, Coordinate};

    const A: f64 = 6378245.0;
    const EE: f64 = 0.006693421622965823;
    const PI: f64 = std::f64::consts::PI;

    // 检查坐标是否在中国范围内
    fn is_in_china_bbox(lon: f64, lat: f64) -> bool {
        lon >= 72.004 && lon <= 137.8347 && lat >= 0.8293 && lat <= 55.8271
    }

    fn transform_lat(x: f64, y: f64) -> f64 {
        let mut ret = -100.0 + 2.0 * x + 3.0 * y + 0.2 * y * y + 0.1 * x * y + 0.2 * x.abs().sqrt();
        ret += ((20.0 * (6f64 * x * PI).sin() + 20.0 * (2.0 * x * PI).sin()) * 2.0) / 3.0;
        ret += ((20.0 * (y * PI).sin() + 40.0 * (y / 3.0 * PI).sin()) * 2.0) / 3.0;
        ret += ((160.0 * (y / 12.0 * PI).sin() + 320.0 * (y * PI / 30.0).sin()) * 2.0) / 3.0;
        ret
    }

    fn transform_lon(x: f64, y: f64) -> f64 {
        let mut ret = 300.0 + x + 2.0 * y + 0.1 * x * x + 0.1 * x * y + 0.1 * x.abs().sqrt();
        ret += ((20.0 * (6f64 * x * PI).sin() + 20.0 * (2.0 * x * PI).sin()) * 2.0) / 3.0;
        ret += ((20.0 * (x * PI).sin() + 40.0 * (x / 3.0 * PI).sin()) * 2.0) / 3.0;
        ret += ((150.0 * (x / 12.0 * PI).sin() + 300.0 * (x / 30.0 * PI).sin()) * 2.0) / 3.0;
        ret
    }

    fn delta(lon: f64, lat: f64) -> (f64, f64) {
        let d_lon = transform_lon(lon - 105.0, lat - 35.0);
        let d_lat = transform_lat(lon - 105.0, lat - 35.0);

        let rad_lat = lat / 180.0 * PI;
        let magic = rad_lat.sin();

        let magic = 1.0 - EE * magic * magic;
        let sqrt_magic = magic.sqrt();

        let d_lon = (d_lon * 180.0) / ((A / sqrt_magic) * rad_lat.cos() * PI);
        let d_lat = (d_lat * 180.0) / (((A * (1.0 - EE)) / (magic * sqrt_magic)) * PI);

        (d_lon, d_lat)
    }

    pub fn wgs84_to_gcj02(coord: Coordinate) -> Result<Coordinate, ConvertError> {
        let (lon, lat) = (coord.lng, coord.lat);

        if !is_in_china_bbox(lon, lat) {
            return Err(ConvertError::OutOfChina);
        }

        let d = delta(lon, lat);

        Ok(Coordinate::new(lon + d.0, lat + d.1))
    }

    pub fn gcj02_to_wgs84(coord: Coordinate) -> Result<Coordinate, ConvertError> {
        let (lon, lat) = (coord.lng, coord.lat);

        if !is_in_china_bbox(lon, lat) {
            return Err(ConvertError::OutOfChina);
        }

        let mut wgs_lon = lon;
        let mut wgs_lat = lat;

        loop {
            let temp_point = wgs84_to_gcj02(Coordinate::new(wgs_lon, wgs_lat));
            if temp_point.is_err() {
                return temp_point;
            }
            let temp_point = temp_point.unwrap();
            let dx = temp_point.lng - lon;
            let dy = temp_point.lat - lat;

            if dx.abs() < 1e-6 && dy.abs() < 1e-6 {
                break;
            }

            wgs_lon -= dx;
            wgs_lat -= dy;
        }

        Ok(Coordinate::new(wgs_lon, wgs_lat))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_gcj02_wgs84() {
        {
            let from = Coordinate::new(114.304569, 30.593354);
            let expected = Coordinate::new(114.310012, 30.590943);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::GCJ02);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let from = Coordinate::new(116.407387, 39.904179);
            let expected = Coordinate::new(116.413629, 39.905582);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::GCJ02);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }

        {
            let from = Coordinate::new(61.972426, 31.998164);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::GCJ02);
            assert!(to.is_err());
        }

        {
            let expected = Coordinate::new(114.304569, 30.593354);
            let from = Coordinate::new(114.310012, 30.590943);
            let to = transform(from, CoordSystem::GCJ02, CoordSystem::WGS84);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let expected = Coordinate::new(116.407387, 39.904179);
            let from = Coordinate::new(116.413629, 39.905582);
            let to = transform(from, CoordSystem::GCJ02, CoordSystem::WGS84);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
    }

    #[test]
    fn test_gcj02_bd09() {
        {
            let from = Coordinate::new(114.304569, 30.593354);
            let expected = Coordinate::new(114.311152, 30.599019);
            let to = transform(from, CoordSystem::GCJ02, CoordSystem::BD09);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let from = Coordinate::new(116.407387, 39.904179);
            let expected = Coordinate::new(116.413772, 39.910501);
            let to = transform(from, CoordSystem::GCJ02, CoordSystem::BD09);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let from = Coordinate::new(61.972426, 31.998164);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::GCJ02);
            assert!(to.is_err());
        }
        {
            let expected = Coordinate::new(114.304569, 30.593354);
            let from = Coordinate::new(114.311152, 30.599019);
            let to = transform(from, CoordSystem::BD09, CoordSystem::GCJ02);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let expected = Coordinate::new(116.407387, 39.904179);
            let from = Coordinate::new(116.413772, 39.910501);
            let to = transform(from, CoordSystem::BD09, CoordSystem::GCJ02);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
    }

    #[test]
    fn test_wgs84_bd09() {
        {
            let expected = Coordinate::new(114.304569, 30.593354);
            let from = Coordinate::new(114.316583, 30.596644);
            let to = transform(from, CoordSystem::BD09, CoordSystem::WGS84);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let expected = Coordinate::new(116.407387, 39.904178);
            let from = Coordinate::new(116.420033, 39.911844);
            let to = transform(from, CoordSystem::BD09, CoordSystem::WGS84);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let from = Coordinate::new(61.972426, 31.998164);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::GCJ02);
            assert!(to.is_err());
        }
        {
            let from = Coordinate::new(114.304569, 30.593354);
            let expected = Coordinate::new(114.316583, 30.596644);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::BD09);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
        {
            let from = Coordinate::new(116.407387, 39.904179);
            let expected = Coordinate::new(116.420033, 39.911844);
            let to = transform(from, CoordSystem::WGS84, CoordSystem::BD09);
            assert!(to.is_ok());
            let to = to.unwrap();
            println!("{} {}", to.lng, to.lat);
            assert!((to.lng - expected.lng).abs() < EPSILON);
            assert!((to.lat - expected.lat).abs() < EPSILON);
        }
    }
}
