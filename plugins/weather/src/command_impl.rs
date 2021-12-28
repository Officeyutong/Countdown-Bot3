
use crate::WeatherPlugin;
use anyhow::anyhow;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::debug;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct LocationSearchResult {
    pub name: String,
    pub id: String,
    pub lat: String,
    pub lon: String,
    pub adm2: String,
    pub adm1: String,
    pub country: String,
    pub tz: String,
    // pub utcOffset: String,
    // pub isDst: String,
    // pub r#type: String,
    // pub rank: String,
}
impl LocationSearchResult {
    pub fn generate_message(&self) -> String {
        format!(
            r###"查询位置: {}, {}, {}, {}
        时区: {}
        经度: {}
        纬度: {}"###,
            self.country, self.adm1, self.adm2, self.name, self.tz, self.lon, self.lat
        )
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RealTimeWeatherResponse {
    // pub cloud: Option<String>,
    // pub dew: Option<String>,
    // pub obsTime: String,
    pub temp: String,
    // #[allow(non_snake_case)]
    pub feels_like: String,
    // pub icon: String,
    pub text: String,
    // pub wind360: String,
    pub wind_dir: String,
    pub wind_scale: String,
    // pub windSpeed: String,
    // pub humidity: String,
    // pub precip: String,
    // pub pressure: String,
    // pub vis: String,
}
impl RealTimeWeatherResponse {
    pub fn generate_message(&self) -> String {
        return format!(
            r###"当前天气: {}
        当前气温: {}摄氏度
        体感温度: {}摄氏度
        风向: {} 风力: {}
        "###,
            self.text, self.temp, self.feels_like, self.wind_dir, self.wind_scale
        );
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ForecastWeatherResponse {
    pub temp_max: String,
    pub temp_min: String,
    pub text_day: String,
    pub text_night: String,
    pub fx_date: String,
}
impl ForecastWeatherResponse {
    pub fn generate_message(&self, day: &str) -> String {
        format!(
            r###"{}天({})
            白天天气: {}
            夜间天气: {}
            最高温度: {}摄氏度
            最低温度: {}摄氏度
            "###,
            day, self.fx_date, self.text_day, self.text_night, self.temp_max, self.temp_min
        )
    }
}
async fn query_location(
    key: &str,
    keyword: &str,
) -> Result<LocationSearchResult, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct Resp {
        pub code: String,
        pub location: Vec<LocationSearchResult>,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://geoapi.qweather.com/v2/city/lookup")
        .query(&[("location", keyword), ("key", key), ("number", "1")])
        .send()
        .await?;
    let recv_str = resp
        .text()
        .await
        .map_err(|e| anyhow!("无法读取文本: {}", e))?
        .clone();
    // .as_str();
    let jsonval = serde_json::from_str::<Resp>(recv_str.as_str())
        .map_err(|e| anyhow!("反序列化时发生错误: {}", e))?;
    if jsonval.code != "200" {
        return Err(anyhow!("Invalid respone code: {}", jsonval.code).into());
    }
    if let Some(o) = jsonval.location.get(0) {
        return Ok(o.clone());
    } else {
        return Err(anyhow!("搜索无结果!").into());
    }
}
async fn query_now_weather(
    key: &str,
    location: &str,
) -> Result<RealTimeWeatherResponse, Box<dyn std::error::Error>> {
    let resp = reqwest::Client::new()
        .get("https://devapi.qweather.com/v7/weather/now")
        .query(&[("location", location), ("key", key), ("number", "1")])
        .send()
        .await?;
    #[derive(Deserialize)]
    struct Resp {
        pub code: String,
        pub now: RealTimeWeatherResponse,
    }
    let parsedval = serde_json::from_str::<Resp>(resp.text().await?.as_str())?;
    if parsedval.code != "200" {
        return Err(anyhow!("Invalid response code: {}", parsedval.code).into());
    }
    return Ok(parsedval.now);
}
async fn query_forecast_weather(
    key: &str,
    location: &str,
) -> ResultType<Vec<ForecastWeatherResponse>> {
    let resp = reqwest::Client::new()
        .get("https://devapi.qweather.com/v7/weather/3d")
        .query(&[("location", location), ("key", key), ("number", "1")])
        .send()
        .await?;
    #[derive(Deserialize)]
    struct Resp {
        pub code: String,
        pub daily: Vec<ForecastWeatherResponse>,
    }
    let val = serde_json::from_str::<Resp>(resp.text().await?.as_str())?;
    if val.code != "200" {
        return Err(anyhow!("Invalid response code: {}", val.code).into());
    }
    return Ok(val.daily);
}
impl WeatherPlugin {
    pub async fn on_weather_command(
        &self,
        _command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("{:#?}",std::thread::current());
        if args.len() == 0 {
            return Err(anyhow!("请输入搜索关键字!").into());
        }
        let key = self.config.as_ref().unwrap().api_key.as_str();
        let location = query_location(key, args[0].as_str()).await?;
        debug!("Location: {:#?}", location);
        let loc_code = &location.id;
        // let (current,forecast) = tokio::try_join!(query_now_weather(key,loc_code),query_forecast_weather(key,loc_code))
        let current = query_now_weather(key, loc_code).await?;
        let forecast = query_forecast_weather(key, loc_code).await?;
        let mut buf = String::new();
        buf.push_str(location.generate_message().as_str());
        buf.push_str("\n");
        buf.push_str(current.generate_message().as_str());
        buf.push_str("\n");
        let days = ["今", "明", "后"];
        for (i, val) in forecast.iter().enumerate() {
            if i >= 3 {
                break;
            }
            buf.push_str(val.generate_message(days[i]).as_str());
            buf.push_str("\n");
        }
        // tokio::time::sleep(Duration::from_secs(100)).await;
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(&sender, buf.as_str())
            .await?;
        return Ok(());
    }
}
