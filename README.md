# OpenMinis Rust

Android 전용 Rust AI 에이전트 프레임워크 — [OpenMinis](https://github.com/OpenMinis/OpenMinis)의 Android 포팅.

## 구조

```
crates/
  core/         에이전트 루프, 툴 디스패치, 세션
  provider/     LLM 프로바이더 (Anthropic, OpenAI, Gemini, ...)
  minis-url/    minis:// URL 엔진 + 세션 파일 관리
  sandbox/      PRoot 샌드박스, 셸, bashism 감지
  speech/       음성 교정 엔진, VAD, 문장 분할
  skills/       스킬 매칭/로드
  memory/       메모리 저장소, 검색
  browser/      브라우저 자동화 로직
  offload/      Android 기능 위임 인터페이스 (trait + registry)
  storage/      SQLite 데이터베이스 (rusqlite)
  scheduled/    예약 작업
  config/       설정 레지스트리
src-tauri/      Tauri Mobile 진입점 (향후)
frontend/       Sycamore/Dioxus UI (WASM, 향후)
```

## 빌드

```sh
cargo build
cargo test
cargo run
```

## 라이선스

GPL-3.0 (원본 OpenMinis와 동일)