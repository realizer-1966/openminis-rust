// 기본 설정 필드 등록

use crate::registry::{ConfigField, ConfigValue, ConfigRisk, ConfigRegistry};

pub fn register_builtins(registry: &mut ConfigRegistry) {
    // 폰트 스케일 — 연속 float (0.70~1.50)
    registry.register(ConfigField {
        path: "chat.inputFontSize".into(),
        display_name: "Input font size".into(),
        description: "Font size multiplier for the chat composer (0.70 to 1.50).".into(),
        value: ConfigValue::Float(1.0),
        default: ConfigValue::Float(1.0),
        risk: ConfigRisk::Normal,
        revertable: true,
    });

    registry.register(ConfigField {
        path: "chat.messageFontSize".into(),
        display_name: "Message font size".into(),
        description: "Font size multiplier for message bodies (0.70 to 1.50).".into(),
        value: ConfigValue::Float(1.0),
        default: ConfigValue::Float(1.0),
        risk: ConfigRisk::Normal,
        revertable: true,
    });

    registry.register(ConfigField {
        path: "appearance.appFontSize".into(),
        display_name: "App font size".into(),
        description: "Font size multiplier for general app text (0.70 to 1.50).".into(),
        value: ConfigValue::Float(1.0),
        default: ConfigValue::Float(1.0),
        risk: ConfigRisk::Normal,
        revertable: true,
    });
}
