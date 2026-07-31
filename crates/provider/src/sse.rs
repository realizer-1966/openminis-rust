// SSE (Server-Sent Events) 스트리밍 파서
// 원본: AIChatViewModel+SSEStream.swift, LLMStreamChunk 관련

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, AsyncRead};
use crate::types::StreamChunk;

/// SSE 스트림을 파싱해서 StreamChunk로 변환
/// 각 프로바이더의 `stream_chat`에서 사용
pub async fn parse_sse_stream<R: AsyncRead + Unpin>(
    reader: R,
    tx: mpsc::Sender<StreamChunk>,
    line_handler: impl Fn(&str) -> Option<StreamChunk> + Send,
) -> Result<()> {
    let mut reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(':') {
            continue; // 빈 줄 또는 코멘트
        }

        if let Some(rest) = trimmed.strip_prefix("data: ") {
            if rest == "[DONE]" {
                let _ = tx.send(StreamChunk::Done).await;
                break;
            }
            if let Some(chunk) = line_handler(rest) {
                let _ = tx.send(chunk).await;
            }
        }
    }

    Ok(())
}