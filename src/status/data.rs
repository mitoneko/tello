/// Telloのステータスデータ
#[derive(Default, Debug, PartialEq, Clone)]
pub struct StatusData {
    pub mid: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub mpry: (i32, i32, i32),
    pub pitch: i32,
    pub roll: i32,
    pub yaw: i32,
    pub vgx: i32,
    pub vgy: i32,
    pub vgz: i32,
    pub templ: i32,
    pub temph: i32,
    pub tof: i32,
    pub h: i32,
    pub bat: u32,
    pub baro: f64,
    pub time: i32,
    pub agx: f64,
    pub agy: f64,
    pub agz: f64,
}

impl StatusData {
    /// 数値文字列を指定の数値型に変換する。
    /// (ステータス解析のユーティリティー)
    fn parse<I>(item: &str, src: &str) -> I
    where
        I: std::str::FromStr + Default,
    {
        item.parse::<I>().unwrap_or_else(|_| {
            eprintln!("field value error:[{}]", src);
            I::default()
        })
    }
}

impl std::str::FromStr for StatusData {
    type Err = TelloStatusParseError;

    /// Telloの受信データの文字列を解析する
    fn from_str(src: &str) -> Result<Self, TelloStatusParseError> {
        let mut ret = Self::default();

        let end = match src.match_indices("\r\n").next() {
            Some((cnt, _)) => cnt,
            None => src.len(),
        };

        for pair in src[0..end].trim().split(';') {
            let item: Vec<&str> = pair.split(':').collect();
            if item.len() == 1 {
                continue;
            } else if item.len() != 2 {
                eprintln!("field format error1: [{}]", pair);
                continue;
            }

            match item[0].trim() {
                "mid" => ret.mid = Self::parse::<i32>(item[1], pair),
                "x" => ret.x = Self::parse::<i32>(item[1], pair),
                "y" => ret.y = Self::parse::<i32>(item[1], pair),
                "z" => ret.z = Self::parse::<i32>(item[1], pair),
                "pitch" => ret.pitch = Self::parse::<i32>(item[1], pair),
                "roll" => ret.roll = Self::parse::<i32>(item[1], pair),
                "yaw" => ret.yaw = Self::parse::<i32>(item[1], pair),
                "vgx" => ret.vgx = Self::parse::<i32>(item[1], pair),
                "vgy" => ret.vgy = Self::parse::<i32>(item[1], pair),
                "vgz" => ret.vgz = Self::parse::<i32>(item[1], pair),
                "templ" => ret.templ = Self::parse::<i32>(item[1], pair),
                "temph" => ret.temph = Self::parse::<i32>(item[1], pair),
                "tof" => ret.tof = Self::parse::<i32>(item[1], pair),
                "h" => ret.h = Self::parse::<i32>(item[1], pair),
                "bat" => ret.bat = Self::parse::<u32>(item[1], pair),
                "baro" => ret.baro = Self::parse::<f64>(item[1], pair),
                "time" => ret.time = Self::parse::<i32>(item[1], pair),
                "agx" => ret.agx = Self::parse::<f64>(item[1], pair),
                "agy" => ret.agy = Self::parse::<f64>(item[1], pair),
                "agz" => ret.agz = Self::parse::<f64>(item[1], pair),
                "mpry" => {
                    let values: Vec<&str> = item[1].split(',').collect();
                    if values.len() == 3 {
                        ret.mpry = (
                            Self::parse::<i32>(values[0], pair),
                            Self::parse::<i32>(values[1], pair),
                            Self::parse::<i32>(values[2], pair),
                        );
                    } else {
                        eprintln!("field format error2: [{}]", pair);
                    }
                }
                _ => {}
            }
        }

        Ok(ret)
    }
}

///ステータス変換用のエラー。実質不使用。
#[derive(Debug, PartialEq, Clone)]
pub struct TelloStatusParseError();

impl std::fmt::Display for TelloStatusParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ステータス解析失敗。でも出るはずがない。")
    }
}
