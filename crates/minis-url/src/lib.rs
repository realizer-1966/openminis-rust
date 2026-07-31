// minis:// URL 엔진 — 세션 단위 리소스 주소 체계
// 원본: docs/specs/minis-url-scheme.md, iOS/Android의 파일 관리 로직

use anyhow::Result;
use thiserror::Error;
use std::path::{Path, PathBuf};

pub mod resolver;
pub mod session_mount;
pub mod namespace;

pub use resolver::UrlResolver;
pub use namespace::Namespace;

#[derive(Debug, Error)]
pub enum MinisUrlError {
    #[error("Invalid minis:// URL: {0}")]
    InvalidUrl(String),
    #[error("Path traversal detected")]
    PathTraversal,
    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),
}
