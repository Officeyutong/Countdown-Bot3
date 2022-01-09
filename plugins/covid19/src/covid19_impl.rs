use crate::{
    common::{get_html, get_inner_text, ThingsThatCanMakeMessage, EXPR},
    COVID19Plugin,
};
use anyhow::anyhow;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct COVIDCityStatEntry {
    pub confirmed_count: i64,
    pub current_confirmed_count: i64,
    pub suspected_count: i64,
    pub cured_count: i64,
    pub high_danger_count: i64,
    pub dead_count: i64,
    pub city_name: String,
}
impl ThingsThatCanMakeMessage for COVIDCityStatEntry {
    fn make_message(&self) -> String {
        return format!(
            "{}: 累计确诊{} 当前确诊{} 疑似{} 治愈{} 高危{} 死亡{}",
            self.city_name,
            self.confirmed_count,
            self.current_confirmed_count,
            self.suspected_count,
            self.cured_count,
            self.high_danger_count,
            self.dead_count
        );
    }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct COVIDProvinceStatEntry {
    pub confirmed_count: i64,
    pub current_confirmed_count: i64,
    pub suspected_count: i64,
    pub cured_count: i64,
    pub high_danger_count: i64,
    pub dead_count: i64,
    pub province_name: String,
    // pub province_short_name: String,
    pub cities: Vec<COVIDCityStatEntry>,
}

impl ThingsThatCanMakeMessage for COVIDProvinceStatEntry {
    fn make_message(&self) -> String {
        return format!(
            "{}: 累计确诊{} 当前确诊{} 疑似{} 治愈{} 高危{} 死亡{}",
            self.province_name,
            self.confirmed_count,
            self.current_confirmed_count,
            self.suspected_count,
            self.cured_count,
            self.high_danger_count,
            self.dead_count
        );
    }
}
impl COVIDProvinceStatEntry {
    pub fn make_city_message(&self) -> String {
        let mut buf = String::new();
        for city in self.cities.iter() {
            buf.push_str(city.make_message().as_str());
            buf.push_str("\n");
        }
        return buf;
    }
}

fn stat_str(v: i64) -> String {
    if v <= 0 {
        format!("({})", v)
    } else {
        format!("(+{})", v)
    }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GeneralStatEntry {
    pub modify_time: i64,
    pub confirmed_incr: i64,
    pub current_confirmed_incr: i64,
    pub suspected_incr: i64,
    pub cured_incr: i64,
    pub serious_incr: i64,
    pub dead_incr: i64,
    pub confirmed_count: i64,
    pub current_confirmed_count: i64,
    pub suspected_count: i64,
    pub cured_count: i64,
    pub serious_count: i64,
    pub dead_count: i64,
}
impl ThingsThatCanMakeMessage for GeneralStatEntry {
    fn make_message(&self) -> String {
        use chrono::prelude::*;
        let modify_time = DateTime::<Local>::from_utc(
            NaiveDateTime::from_timestamp(self.modify_time / 1000, 0),
            *Local.timestamp(0, 0).offset(),
        );
        let time_str = modify_time.format("%Y-%m-%d %H:%M:%S").to_string();
        return format!(
            "累计确诊{}{} | 当前确诊{}{} | 疑似{}{} | 治愈{}{} | 重症{}{} | 死亡{}{}\n更新于{}",
            self.confirmed_count,
            stat_str(self.confirmed_incr),
            self.current_confirmed_count,
            stat_str(self.current_confirmed_incr),
            self.suspected_count,
            stat_str(self.suspected_incr),
            self.cured_count,
            stat_str(self.cured_incr),
            self.serious_count,
            stat_str(self.serious_incr),
            self.dead_count,
            stat_str(self.dead_incr),
            time_str
        );
    }
}

impl COVID19Plugin {
    pub async fn handle_covid19(
        &self,
        province: Option<&str>,
        sender: &SenderType,
    ) -> ResultType<()> {
        let result = {
            let html = get_html().await?;
            let script_text = get_inner_text(&html, "#getAreaStat")?;
            let search_result = EXPR
                .find(&script_text)
                .ok_or(anyhow!("未找到区域统计对应的JSON!"))?;
            let region_stat =
                serde_json::from_str::<Vec<COVIDProvinceStatEntry>>(search_result.as_str())?;
            let mut buf = String::from("数据来源: 丁香医生\n");

            if let Some(province) = province {
                let mut province_inst: Option<&COVIDProvinceStatEntry> = None;
                for prov in region_stat.iter() {
                    if prov.province_name.contains(province) {
                        province_inst = Some(prov);
                    }
                }
                if let Some(item) = province_inst {
                    buf.push_str(format!("{}的新冠肺炎疫情情况:\n", item.province_name).as_str());
                    buf.push_str(item.make_message().as_str());
                    buf.push('\n');
                    buf.push_str(item.make_city_message().as_str());
                } else {
                    return Err(anyhow!("非法省份: {}", province).into());
                }
            } else {
                lazy_static! {
                    static ref EXPR_BRACKET: Regex = Regex::new(r"= ?(\{.*\})\}catch").unwrap();
                }
                let global_stat = get_inner_text(&html, "#getStatisticsService")?;
                debug!("{}", global_stat);
                let group_str = EXPR_BRACKET
                    .captures(&global_stat)
                    .ok_or(anyhow!("未找到相应JSON内容!"))?
                    .get(1)
                    .ok_or(anyhow!("无法获取匹配分组具体内容!"))?
                    .as_str();
                info!("{}", group_str);
                let global_stat_obj = serde_json::from_str::<GeneralStatEntry>(group_str)?;
                buf.push_str(global_stat_obj.make_message().as_str());
                buf.push_str("\n");
                for prov in region_stat.iter() {
                    buf.push_str(prov.make_message().as_str());
                    buf.push_str("\n");
                }
            }
            buf
        };
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(sender, result.as_str())
            .await?;
        return Ok(());
    }
}
