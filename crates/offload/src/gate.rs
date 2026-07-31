// 권한 게이트 — tri-state (Allowed / AskOnce / NotAllowed)
// 원본: OffloadGate.kt, OffloadPermissionManager.kt

use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

/// 권한 확인 — 핸들러의 check_permission 결과에 따라 게이트
pub struct OffloadGate;

impl OffloadGate {
    /// 권한 확인 후, 거부되면 PERMISSION_DENIED 결과 반환
    pub fn enforce(handler: &dyn OffloadHandler, session_id: Option<&str>) -> Option<OffloadResult> {
        match handler.check_permission(session_id) {
            PermissionState::Allowed => None,
            PermissionState::AskOnce => {
                // TODO: 사용자에게 묻기 (JNI経由)
                // 현재는 허용으로 처리
                None
            }
            PermissionState::NotAllowed => {
                Some(OffloadResult::permission_denied(handler.display_name()))
            }
        }
    }
}