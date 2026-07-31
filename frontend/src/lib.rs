// OpenMinis Rust — 프론트엔드 진입점 (Sycamore → WASM)
// Tauri Mobile의 WebView에서 실행되어 Rust 코어와 통신
//
// Tauri IPC를 통해 Rust 코어의 에이전트 기능을 호출:
// - 메시지 전송 (send_message)
// - 세션 목록 (list_sessions)
// - 스트리밍 토큰 수신 (agent:token 이벤트)
// - 툴 실행 상태 (tool:started, tool:finished)

use sycamore::prelude::*;
use serde::{Deserialize, Serialize};

// ── Tauri IPC 브릿지 ──
// Tauri의 invoke() 함수를 호출하여 Rust 코어와 통신
// 실제 빌드에서는 @tauri-apps/api의 invoke를 사용

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    minis_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionSummary {
    id: String,
    title: String,
}

// ── 메인 앱 ──

#[component]
fn App() -> View {
    let messages = create_signal(Vec::<ChatMessage>::new());
    let input_text = create_signal(String::new());
    let is_streaming = create_signal(false);
    let sessions = create_signal(Vec::<SessionSummary>::new());
    let current_session = create_signal(String::new());

    // 메시지 전송
    let send_message = move |_| {
        let text = input_text.get().as_ref().clone();
        if text.is_empty() { return; }

        // 사용자 메시지 추가
        messages.update(|msgs| {
            msgs.push(ChatMessage {
                role: "user".into(),
                content: text.clone(),
                minis_url: None,
            });
        });

        input_text.set(String::new());
        is_streaming.set(true);

        // TODO: Tauri invoke("send_message", { message: text })
        // TODO: Tauri listen("agent:token") → messages.update
        // TODO: Tauri listen("agent:done") → is_streaming.set(false)

        // 임시 placeholder 응답
        is_streaming.set(false);
        messages.update(|msgs| {
            msgs.push(ChatMessage {
                role: "assistant".into(),
                content: "[Rust 코어와 연결되면 여기에 응답이 표시됩니다]".into(),
                minis_url: None,
            });
        });
    };

    // 세션 목록 로드
    let load_sessions = move |_| {
        // TODO: Tauri invoke("list_sessions")
        sessions.set(vec![
            SessionSummary { id: "s1".into(), title: "새 대화".into() },
        ]);
        current_session.set("s1".into());
    };

    view! {
        div(class="app") {
            // 사이드바 — 세션 목록
            aside(class="sidebar") {
                button(class="new-chat-btn", on:click=load_sessions) { "새 대화" }
                ul(class="session-list") {
                    Keyed(
                        iterable=sessions,
                        view=move |session| {
                            view! {
                                li(class="session-item") {
                                    (session.title)
                                }
                            }
                        },
                        key=|session| session.id.clone(),
                    )
                }
            }

            // 메인 — 채팅 영역
            main(class="chat-main") {
                // 메시지 목록
                div(class="message-list") {
                    Keyed(
                        iterable=messages,
                        view=move |msg| {
                            let is_user = msg.role == "user";
                            view! {
                                div(class=if is_user { "message user" } else { "message assistant" }) {
                                    (msg.content)
                                }
                            }
                        },
                        key=|msg| format!("{}-{}", msg.role, msg.content.len()),
                    )
                }

                // 입력 바
                div(class="input-bar") {
                    input(
                        class="message-input",
                        type="text",
                        placeholder="메시지를 입력하세요...",
                        bind:value=input_text,
                        on:keydown=move |e| {
                            if e.key() == "Enter" && !e.shift_key() {
                                e.prevent_default();
                                send_message(e);
                            }
                        },
                    )
                    button(class="send-btn", on:click=send_message) { "전송" }
                }
            }
        }
    }
}

fn main() {
    sycamore::render(|| view! { App() });
}