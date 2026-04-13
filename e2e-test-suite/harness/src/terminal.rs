use anyhow::{bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

/// Live terminal session connected to ttyd via WebSocket.
///
/// Each session corresponds to one ttyd process (and therefore one app
/// invocation). Dropping the session closes the WebSocket, which causes
/// ttyd to terminate the app process.
pub struct TerminalSession {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    parser: vt100::Parser,
}

impl TerminalSession {
    /// Opens a WebSocket connection to the given ttyd `ws://` URL and sends
    /// an initial terminal-size message.
    pub async fn connect(url: &str, cols: u16, rows: u16) -> Result<Self> {
        let (ws, _response) = connect_async(url)
            .await
            .with_context(|| format!("Connecting to ttyd at {url}"))?;

        let mut session = Self {
            ws,
            parser: vt100::Parser::new(rows, cols, 0),
        };

        // Tell ttyd the terminal dimensions
        session.send_resize(cols, rows).await?;

        // Give the process a moment to start and produce initial output
        session.drain(Duration::from_millis(500)).await?;

        Ok(session)
    }

    /// Waits until the terminal screen contains `text`, or fails after `timeout`.
    pub async fn wait_for_text(&mut self, text: &str, timeout: Duration) -> Result<()> {
        let start = tokio::time::Instant::now();
        loop {
            self.drain(Duration::from_millis(100)).await?;

            if self.screen_contains(text) {
                return Ok(());
            }

            if start.elapsed() > timeout {
                bail!(
                    "Timeout ({timeout:?}) waiting for text '{text}'.\n\
                     Current screen contents:\n{}",
                    self.screen_text()
                );
            }
        }
    }

    /// Returns the full textual content currently visible on the virtual screen.
    pub fn screen_text(&self) -> String {
        self.parser.screen().contents()
    }

    /// Returns true if `text` appears anywhere on the current screen.
    pub fn screen_contains(&self, text: &str) -> bool {
        self.parser.screen().contents().contains(text)
    }

    /// Sends a keystroke (or arbitrary bytes) to the app via ttyd.
    pub async fn send_input(&mut self, input: &str) -> Result<()> {
        let mut payload = vec![0u8]; // ttyd client→server type: INPUT
        payload.extend_from_slice(input.as_bytes());
        self.ws
            .send(Message::Binary(payload.into()))
            .await
            .context("Sending input to ttyd")?;
        Ok(())
    }

    /// Sends a terminal resize message.
    async fn send_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        let json = serde_json::json!({"columns": cols, "rows": rows});
        let mut payload = vec![1u8]; // ttyd client→server type: RESIZE_TERMINAL
        payload.extend_from_slice(json.to_string().as_bytes());
        self.ws
            .send(Message::Binary(payload.into()))
            .await
            .context("Sending resize to ttyd")?;
        Ok(())
    }

    /// Reads all available WebSocket messages until the stream is idle for
    /// `idle_timeout`, feeding any terminal output into the vt100 parser.
    async fn drain(&mut self, idle_timeout: Duration) -> Result<()> {
        loop {
            match tokio::time::timeout(idle_timeout, self.ws.next()).await {
                // Received a message
                Ok(Some(Ok(msg))) => self.handle_message(msg),
                // WebSocket error
                Ok(Some(Err(e))) => bail!("WebSocket error: {e}"),
                // Stream closed
                Ok(None) => bail!("WebSocket stream closed unexpectedly"),
                // Timeout — no more data for now
                Err(_) => return Ok(()),
            }
        }
    }

    fn handle_message(&mut self, msg: Message) {
        let data = match msg {
            Message::Binary(b) => b.to_vec(),
            // ttyd may also send text frames in some versions
            Message::Text(t) => t.as_bytes().to_vec(),
            _ => return,
        };

        if data.is_empty() {
            return;
        }

        // ttyd server→client protocol: first byte is the message type
        match data[0] {
            0 => {
                // OUTPUT — feed terminal data to vt100
                self.parser.process(&data[1..]);
            }
            1 => {
                // SET_WINDOW_TITLE — ignored
            }
            2 => {
                // SET_PREFERENCES — ignored
            }
            _ => {
                log::debug!("Unknown ttyd message type: {}", data[0]);
            }
        }
    }
}
