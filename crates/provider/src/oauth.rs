// OAuth 토큰 갱신 — Anthropic OAuth, OpenAI OAuth 등
// 원본: OAuthRefreshCoordinator.swift, OAuthRefreshSingleFlight.swift
// (Android에는 별도 OAuth 파일이 없으므로 iOS 구조 참조)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn will_expire_within(&self, secs: i64) -> bool {
        Utc::now() + chrono::Duration::seconds(secs) > self.expires_at
    }
}

pub struct OAuthRefreshCoordinator {
    // TODO: refresh_token을 사용한 토큰 갱신 구현
}

impl OAuthRefreshCoordinator {
    pub async fn refresh(
        &self,
        _refresh_token: &str,
        _client_id: &str,
        _token_url: &str,
    ) -> Result<OAuthToken> {
        todo!("OAuth token refresh implementation")
    }
}