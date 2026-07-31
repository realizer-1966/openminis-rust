// OpenMinis Rust — 메인 진입점
// 로컬 테스트 서버 모드로 실행

#[tokio::main]
async fn main() -> std::io::Result<()> {
    crate::server::run_server().await
}

mod server;