// minis:// 네임스페이스 정의
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Namespace {
    Attachments,
    Workspace,
    Offloads,
    Browser,
    Shared,
    Memory,
    Mounts,
}

impl Namespace {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "attachments" => Some(Self::Attachments),
            "workspace" => Some(Self::Workspace),
            "offloads" => Some(Self::Offloads),
            "browser" => Some(Self::Browser),
            "shared" => Some(Self::Shared),
            "memory" => Some(Self::Memory),
            "mounts" => Some(Self::Mounts),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Attachments => "attachments",
            Self::Workspace => "workspace",
            Self::Offloads => "offloads",
            Self::Browser => "browser",
            Self::Shared => "shared",
            Self::Memory => "memory",
            Self::Mounts => "mounts",
        }
    }

    pub fn linux_prefix(&self) -> &'static str {
        match self {
            Self::Attachments => "/var/minis/attachments",
            Self::Workspace => "/var/minis/workspace",
            Self::Offloads => "/var/minis/offloads",
            Self::Browser => "/var/minis/browser",
            Self::Shared => "/var/minis/shared",
            Self::Memory => "/var/minis/memory",
            Self::Mounts => "/var/minis/mounts",
        }
    }
}
