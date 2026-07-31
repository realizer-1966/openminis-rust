// 세션 파일 마운트/하베스트 — 세션 전환 시 파일 교체
// 원본: iOS/Android의 session mount/unmount 로직

use std::path::PathBuf;
use crate::namespace::Namespace;

/// 세션 파일 관리자 — 세션 전환 시:
/// 1. Harvest: 현재 세션의 변경 파일을 영구 저장소로 복사
/// 2. Clear: /var/minis/ 디렉토리 정리
/// 3. Mount: 새 세션의 파일을 /var/minis/로 복사
pub struct SessionMount {
    /// 영구 저장소 루트 (예: /data/data/com.openminis.app/files/sessions/<id>/)
    persistent_root: PathBuf,
    /// Linux 가시 경로 (항상 /var/minis/)
    linux_root: PathBuf,
}

impl SessionMount {
    pub fn new(persistent_root: PathBuf) -> Self {
        Self {
            persistent_root,
            linux_root: PathBuf::from("/var/minis"),
        }
    }

    pub fn session_dir(&self, session_id: &str) -> PathBuf {
        self.persistent_root.join(session_id)
    }

    /// 세션 마운트 — 영구 저장소에서 /var/minis/로 파일 복사
    pub fn mount(&self, session_id: &str) -> std::io::Result<()> {
        let src = self.session_dir(session_id);
        if !src.exists() {
            std::fs::create_dir_all(&src)?;
            return Ok(());
        }
        // 각 네임스페이스 디렉토리 복사
        for ns in [Namespace::Attachments, Namespace::Workspace, Namespace::Offloads, Namespace::Browser] {
            let ns_src = src.join(ns.as_str());
            let ns_dst = self.linux_root.join(ns.as_str());
            if ns_src.exists() {
                copy_dir_recursive(&ns_src, &ns_dst)?;
            } else {
                std::fs::create_dir_all(&ns_dst)?;
            }
        }
        Ok(())
    }

    /// 세션 하베스트 — /var/minis/의 파일을 영구 저장소로 복사
    pub fn harvest(&self, session_id: &str) -> std::io::Result<()> {
        let dst = self.session_dir(session_id);
        std::fs::create_dir_all(&dst)?;
        for ns in [Namespace::Attachments, Namespace::Workspace, Namespace::Offloads, Namespace::Browser] {
            let ns_src = self.linux_root.join(ns.as_str());
            let ns_dst = dst.join(ns.as_str());
            if ns_src.exists() {
                copy_dir_recursive(&ns_src, &ns_dst)?;
            }
        }
        Ok(())
    }

    /// 세션 삭제 — 영구 저장소의 세션 디렉토리 제거
    pub fn delete(&self, session_id: &str) -> std::io::Result<()> {
        let dir = self.session_dir(session_id);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_recursive(&path, &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(&path, &dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
