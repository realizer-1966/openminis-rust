// 날씨 offload — Open-Meteo API (API 키 불필요)
// 원본: WeatherOffloadHandler.kt, WeatherManager.kt
// Rust에서 직접 HTTP 호출 가능 (JNI 불필요)

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct WeatherHandler {
    client: reqwest::Client,
}

impl WeatherHandler {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }

    async fn fetch_meteo(&self, lat: f64, lon: f64, params: &str) -> Result<Value> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&{}",
            lat, lon, params
        );
        let response = self.client.get(&url).send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }
}

const HELP: &str = r#"android-weather — fetch weather from Open-Meteo (no API key needed).

Usage:
  android-weather current   --lat L --lon N
  android-weather hourly    --lat L --lon N [--hours N]
  android-weather daily     --lat L --lon N [--days N]
  android-weather alerts    --lat L --lon N
  android-weather report    --lat L --lon N [--hours N] [--days N]
"#;

#[async_trait]
impl OffloadHandler for WeatherHandler {
    fn name(&self) -> &str { "weather" }
    fn display_name(&self) -> &str { "android-weather" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("current");

        let lat = args.get("lat").and_then(|v| v.as_f64());
        let lon = args.get("lon").or_else(|| args.get("lng")).and_then(|v| v.as_f64());

        if lat.is_none() || lon.is_none() {
            return Ok(OffloadResult::bad_args(&format!("{}: --lat and --lon are required", sub)));
        }

        let lat = lat.unwrap();
        let lon = lon.unwrap();

        if !(-90.0..=90.0).contains(&lat) {
            return Ok(OffloadResult::bad_args("latitude out of range (-90..90)"));
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Ok(OffloadResult::bad_args("longitude out of range (-180..180)"));
        }

        let params = match sub {
            "current" => "current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m,wind_direction_10m",
            "hourly" => {
                let hours = args.get("hours").and_then(|v| v.as_u64()).unwrap_or(24);
                &format!("hourly=temperature_2m,precipitation_probability,weather_code&forecast_hours={}", hours.min(168))
            }
            "daily" => {
                let days = args.get("days").and_then(|v| v.as_u64()).unwrap_or(7);
                &format!("daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability,wind_speed_10m_max&forecast_days={}", days.min(16))
            }
            "report" => "current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m&hourly=temperature_2m,precipitation_probability&daily=weather_code,temperature_2m_max,temperature_2m_min&forecast_hours=24&forecast_days=7",
            "alerts" => "current=temperature_2m,weather_code",
            _ => return Ok(OffloadResult::bad_args(&format!("unknown subcommand '{}'", sub))),
        };

        match self.fetch_meteo(lat, lon, params).await {
            Ok(data) => Ok(OffloadResult::ok(data)),
            Err(e) => Ok(OffloadResult::error(&format!("weather API error: {}", e), 1)),
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for WeatherHandler {
    fn default() -> Self { Self::new() }
}