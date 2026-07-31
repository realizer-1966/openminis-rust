// 세션 파일 마운트/하베스트 — 세션 전환 시 파일 교체
// 원본: iOS/Android의 session mount/unmount 로직, docs/specs/minis-url-scheme.md §5
//
// 세션 전환 라이프사이클:
// 1. Harvest: 현재 세션의 변경 파일을 영구 저장소로 복사
// 2. Clear: /var/minis/ 디렉토리 정리
// 3. Mount: 새 세션의 파일을 /var/minis/로 복사
// 4. Ensure: 모든 네임스페이스 디렉토리 존재 보장

use std::path::{Path, PathBuf};
use std::collections::HashSet;
use crate::namespace::Namespace;
use crate::resolver::UrlResolver;

/// 마운트 대상 네임스페이스 — shared, memory, mounts는 세션 격리에서 제외
const MOUNTABLE_NAMESPACES: [Namespace; 4] = [
    Namespace::Attachments,
    Namespace::Workspace,
    Namespace::Offloads,
    Namespace::Browser,
];

/// 세션 파일 관리자
pub struct SessionMount {
    /// 영구 저장소 루트 (예: /data/data/com.openminis.app/files/sessions/)
    persistent_root: PathBuf,
    /// Linux 가시 경로 (항상 /var/minis/)
    linux_root: PathBuf,
}

/// 세션 전환 결과
#[derive(Debug)]
pub struct SwitchResult {
    pub harvested_session: Option<String>,
    pub mounted_session: String,
    pub files_harvested: usize,
    pub files_mounted: usize,
}

impl SessionMount {
    pub fn new(persistent_root: PathBuf) -> Self {
        Self {
            persistent_root,
            linux_root: PathBuf::from("/var/minis"),
        }
    }

    pub fn with_linux_root(mut self, root: PathBuf) -> Self {
        self.linux_root = root;
        self
    }

    pub fn session_dir(&self, session_id: &str) -> PathBuf {
        self.persistent_root.join(session_id)
    }

    /// 모든 네임스페이스 디렉토리가 존재하도록 보장
    pub fn ensure_namespaces(&self) -> std::io::Result<()> {
        for ns in MOUNTABLE_NAMESPACES {
            std::fs::create_dir_all(self.linux_root.join(ns.as_str()))?;
        }
        Ok(())
    }

    /// 세션 마운트 — 영구 저장소에서 /var/minis/로 파일 복사
    pub fn mount(&self, session_id: &str) -> std::io::Result<usize> {
        self.ensure_namespaces()?;
        let src = self.session_dir(session_id);
        if !src.exists() {
            std::fs::create_dir_all(&src)?;
            // 빈 네임스페이스 디렉토리 생성
            for ns in MOUNTABLE_NAMESPACES {
                std::fs::create_dir_all(self.linux_root.join(ns.as_str()))?;
            }
            return Ok(0);
        }

        let mut count = 0;
        for ns in MOUNTABLE_NAMESPACES {
            let ns_src = src.join(ns.as_str());
            let ns_dst = self.linux_root.join(ns.as_str());
            if ns_src.exists() {
                count += copy_dir_recursive(&ns_src, &ns_dst)?;
            } else {
                std::fs::create_dir_all(&ns_dst)?;
            }
        }
        Ok(count)
    }

    /// 세션 하베스트 — /var/minis/의 파일을 영구 저장소로 복사
    pub fn harvest(&self, session_id: &str) -> std::io::Result<usize> {
        let dst = self.session_dir(session_id);
        std::fs::create_dir_all(&dst)?;

        let mut count = 0;
        for ns in MOUNTABLE_NAMESPACES {
            let ns_src = self.linux_root.join(ns.as_str());
            let ns_dst = dst.join(ns.as_str());
            if ns_src.exists() {
                count += copy_dir_recursive(&ns_src, &ns_dst)?;
            }
        }
        Ok(count)
    }

    /// /var/minis/ 디렉토리 정리 — 마운트 가능한 네임스페이스만
    pub fn clear(&self) -> std::io::Result<()> {
        for ns in MOUNTABLE_NAMESPACES {
            let ns_dir = self.linux_root.join(ns.as_str());
            if ns_dir.exists() {
                std::fs::remove_dir_all(&ns_dir)?;
            }
            std::fs::create_dir_all(&ns_dir)?;
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

    /// 세션 전환 — harvest → clear → mount 원자적 수행
    pub fn switch(
        &self,
        from_session: Option<&str>,
        to_session: &str,
    ) -> std::io::Result<SwitchResult> {
        // 1. Harvest: 현재 세션의 파일을 영구 저장소로
        let files_harvested = if let Some(from) = from_session {
            self.harvest(from)?
        } else {
            0
        };

        // 2. Clear: /var/minis/ 정리
        self.clear()?;

        // 3. Mount: 새 세션 파일 로드
        let files_mounted = self.mount(to_session)?;

        Ok(SwitchResult {
            harvested_session: from_session.map(|s| s.to_string()),
            mounted_session: to_session.to_string(),
            files_harvested,
            files_mounted,
        })
    }

    /// 세션의 디스크 사용량 (바이트)
    pub fn session_size(&self, session_id: &str) -> u64 {
        let dir = self.session_dir(session_id);
        dir_size(&dir)
    }

    /// 모든 세션 ID 목록 (영구 저장소의 디렉토리 이름)
    pub fn list_sessions(&self) -> Vec<String> {
        let mut sessions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.persistent_root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        sessions.push(name.to_string());
                    }
                }
            }
        }
        sessions.sort();
        sessions
    }

    /// /var/minis/ 내 파일 목록을 minis:// URL로 변환
    pub fn list_minis_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();
        for ns in MOUNTABLE_NAMESPACES {
            let ns_dir = self.linux_root.join(ns.as_str());
            if ns_dir.exists() {
                collect_urls_recursive(&ns_dir, ns.as_str(), &mut urls);
            }
        }
        urls.sort();
        urls
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(dst)?;
    let mut count = 0;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            count += copy_dir_recursive(&path, &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(&path, &dst.join(entry.file_name()))?;
            count += 1;
        }
    }
    Ok(count)
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn collect_urls_recursive(dir: &Path, ns: &str, urls: &mut Vec<String>) {
    let prefix = format!("/var/minis/{}/", ns);
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_urls_recursive(&path, ns, urls);
            } else if let Some(path_str) = path.to_str() {
                // 상대 경로 추출 — ns 디렉토리 이후 경로
                // /var/minis/<ns>/<relative> → minis://<ns>/<relative>
                // 또는 임의의 linux_root/<ns>/<relative>
                if let Some(idx) = path_str.find(&format!("/{}/", ns)) {
                    let relative = &path_str[idx + ns.len() + 2..]; // skip "/<ns>/"
                    if !relative.is_empty() && !relative.contains("..") {
                        urls.push(format!("minis://{}/{}", ns, relative));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("minis-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_mount_empty_session() {
        let persistent = temp_dir();
        let linux_root = temp_dir();
        let mount = SessionMount::new(persistent.clone())
            .with_linux_root(linux_root.clone());

        let count = mount.mount("session-1").unwrap();
        assert_eq!(count, 0);
        // 네임스페이스 디렉토리 생성 확인
        assert!(linux_root.join("attachments").exists());
        assert!(linux_root.join("workspace").exists());
    }

    #[test]
    fn test_harvest_and_mount() {
        let persistent = temp_dir();
        let linux_root = temp_dir();
        let mount = SessionMount::new(persistent.clone())
            .with_linux_root(linux_root.clone());

        // linux_root에 파일 생성
        std::fs::create_dir_all(linux_root.join("workspace")).unwrap();
        std::fs::write(linux_root.join("workspace/test.txt"), "hello").unwrap();
        std::fs::create_dir_all(linux_root.join("attachments")).unwrap();
        std::fs::write(linux_root.join("attachments/photo.png"), "PNG").unwrap();

        // harvest → 영구 저장소로
        let harvested = mount.harvest("session-1").unwrap();
        assert_eq!(harvested, 2);
        assert!(persistent.join("session-1/workspace/test.txt").exists());
        assert!(persistent.join("session-1/attachments/photo.png").exists());

        // clear → /var/minis/ 정리
        mount.clear().unwrap();
        assert!(!linux_root.join("workspace/test.txt").exists());

        // mount → 영구 저장소에서 복원
        let mounted = mount.mount("session-1").unwrap();
        assert_eq!(mounted, 2);
        assert!(linux_root.join("workspace/test.txt").exists());
        assert_eq!(std::fs::read_to_string(linux_root.join("workspace/test.txt")).unwrap(), "hello");
    }

    #[test]
    fn test_switch() {
        let persistent = temp_dir();
        let linux_root = temp_dir();
        let mount = SessionMount::new(persistent.clone())
            .with_linux_root(linux_root.clone());

        // session-1 파일 생성
        std::fs::create_dir_all(persistent.join("session-1/workspace")).unwrap();
        std::fs::write(persistent.join("session-1/workspace/file1.txt"), "content1").unwrap();

        // session-2 파일 생성
        std::fs::create_dir_all(persistent.join("session-2/workspace")).unwrap();
        std::fs::write(persistent.join("session-2/workspace/file2.txt"), "content2").unwrap();

        // session-1 마운트
        mount.mount("session-1").unwrap();
        assert!(linux_root.join("workspace/file1.txt").exists());

        // linux_root에 새 파일 추가
        std::fs::write(linux_root.join("workspace/file3.txt"), "content3").unwrap();

        // session-1 → session-2 전환
        let result = mount.switch(Some("session-1"), "session-2").unwrap();
        assert_eq!(result.files_harvested, 2); // file1 + file3
        assert_eq!(result.files_mounted, 1); // file2

        // session-1의 file3이 harvest되었는지 확인
        assert!(persistent.join("session-1/workspace/file3.txt").exists());

        // session-2의 file2가 mount되었는지 확인
        assert!(linux_root.join("workspace/file2.txt").exists());
        assert!(!linux_root.join("workspace/file1.txt").exists());
    }

    #[test]
    fn test_delete_session() {
        let persistent = temp_dir();
        let mount = SessionMount::new(persistent.clone());

        std::fs::create_dir_all(persistent.join("session-1/workspace")).unwrap();
        std::fs::write(persistent.join("session-1/workspace/file.txt"), "data").unwrap();

        mount.delete("session-1").unwrap();
        assert!(!persistent.join("session-1").exists());
    }

    #[test]
    fn test_list_sessions() {
        let persistent = temp_dir();
        let mount = SessionMount::new(persistent.clone());

        std::fs::create_dir_all(persistent.join("session-a")).unwrap();
        std::fs::create_dir_all(persistent.join("session-b")).unwrap();
        std::fs::write(persistent.join("not-a-dir.txt"), "ignore me").unwrap();

        let sessions = mount.list_sessions();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session-a".to_string()));
        assert!(sessions.contains(&"session-b".to_string()));
    }

    #[test]
    fn test_session_size() {
        let persistent = temp_dir();
        let mount = SessionMount::new(persistent.clone());

        std::fs::create_dir_all(persistent.join("s1/workspace")).unwrap();
        std::fs::write(persistent.join("s1/workspace/big.txt"), "A".repeat(1000)).unwrap();

        let size = mount.session_size("s1");
        assert!(size >= 1000);
    }

    #[test]
    fn test_list_minis_urls() {
        let persistent = temp_dir();
        let linux_root = temp_dir();
        let mount = SessionMount::new(persistent)
            .with_linux_root(linux_root.clone());

        std::fs::create_dir_all(linux_root.join("workspace")).unwrap();
        std::fs::write(linux_root.join("workspace/report.csv"), "data").unwrap();
        std::fs::create_dir_all(linux_root.join("attachments")).unwrap();
        std::fs::write(linux_root.join("attachments/photo.png"), "PNG").unwrap();

        let urls = mount.list_minis_urls();
        assert!(urls.contains(&"minis://workspace/report.csv".to_string()));
        assert!(urls.contains(&"minis://attachments/photo.png".to_string()));
    }

    #[test]
    fn test_clear_preserves_directories() {
        let persistent = temp_dir();
        let linux_root = temp_dir();
        let mount = SessionMount::new(persistent)
            .with_linux_root(linux_root.clone());

        std::fs::create_dir_all(linux_root.join("workspace")).unwrap();
        std::fs::write(linux_root.join("workspace/file.txt"), "data").unwrap();

        mount.clear().unwrap();
        // 파일은 삭제되지만 디렉토리는 유지
        assert!(linux_root.join("workspace").exists());
        assert!(!linux_root.join("workspace/file.txt").exists());
    }
}