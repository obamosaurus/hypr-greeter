use greetd_ipc::{AuthMessageType, Request, Response};
use std::fmt;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Typed error for greetd operations
#[derive(Debug)]
pub enum GreetdError {
    ConnectionFailed(String),
    AuthFailed(String),
    SessionFailed(String),
    Protocol(String),
}

impl fmt::Display for GreetdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GreetdError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            GreetdError::AuthFailed(msg) => write!(f, "Authentication failed: {}", msg),
            GreetdError::SessionFailed(msg) => write!(f, "Session failed: {}", msg),
            GreetdError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for GreetdError {}

/// Result type for greetd operations
pub type GreetdResult<T> = Result<T, GreetdError>;

/// greetd client for authentication
pub struct GreetdClient {
    stream: UnixStream,
}

impl GreetdClient {
    /// Connect to greetd daemon
    pub async fn connect() -> GreetdResult<Self> {
        let socket_path = std::env::var("GREETD_SOCK")
            .unwrap_or_else(|_| "/run/greetd.sock".to_string());

        let stream = UnixStream::connect(&socket_path).await.map_err(|e| {
            GreetdError::ConnectionFailed(format!("{}: {}", socket_path, e))
        })?;
        Ok(Self { stream })
    }

    /// Authenticate a user with password
    pub async fn authenticate(
        &mut self,
        username: &str,
        password: &str,
    ) -> GreetdResult<()> {
        let request = Request::CreateSession {
            username: username.to_string(),
        };
        self.send_request(request).await?;

        match self.read_response().await? {
            Response::AuthMessage { auth_message_type, .. } => {
                match auth_message_type {
                    AuthMessageType::Secret { .. } => {
                        self.send_password(password).await?;
                    }
                    _ => return Err(GreetdError::Protocol("Unexpected auth message type".into())),
                }
            }
            Response::Error { error_type, description } => {
                return Err(GreetdError::AuthFailed(
                    format!("{:?}: {}", error_type, description),
                ));
            }
            _ => return Err(GreetdError::Protocol("Unexpected response during auth".into())),
        }

        Ok(())
    }

    /// Send password response
    async fn send_password(&mut self, password: &str) -> GreetdResult<()> {
        let request = Request::PostAuthMessageResponse {
            response: Some(password.to_string()),
        };
        self.send_request(request).await?;

        match self.read_response().await? {
            Response::Success => Ok(()),
            Response::Error { error_type, description } => {
                Err(GreetdError::AuthFailed(
                    format!("{:?}: {}", error_type, description),
                ))
            }
            _ => Err(GreetdError::Protocol("Unexpected response after password".into())),
        }
    }

    /// Start a session with the specified command
    pub async fn start_session(&mut self, command: &str) -> GreetdResult<()> {
        let cmd_parts: Vec<String> = command
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if cmd_parts.is_empty() {
            return Err(GreetdError::SessionFailed("Empty session command".into()));
        }

        let request = Request::StartSession {
            cmd: cmd_parts,
            env: vec![],
        };

        self.send_request(request).await?;

        // greetd may exec into the session before responding — that's success.
        // Try to read a response with a short timeout.
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            self.read_response(),
        ).await {
            Ok(Ok(Response::Success)) => Ok(()),
            Ok(Ok(Response::Error { error_type, description })) => {
                Err(GreetdError::SessionFailed(
                    format!("{:?}: {}", error_type, description),
                ))
            }
            // Timeout or connection closed = greetd exec'd, which is success
            Ok(Err(_)) | Err(_) => Ok(()),
            Ok(Ok(_)) => Err(GreetdError::Protocol("Unexpected response to start_session".into())),
        }
    }

    /// Cancel the current session
    pub async fn cancel_session(&mut self) -> GreetdResult<()> {
        let request = Request::CancelSession;
        self.send_request(request).await?;
        match self.read_response().await? {
            Response::Success => Ok(()),
            Response::Error { error_type, description } => {
                Err(GreetdError::SessionFailed(
                    format!("Cancel failed: {:?} - {}", error_type, description),
                ))
            }
            _ => Err(GreetdError::Protocol("Unexpected response to cancel".into())),
        }
    }

    /// Send a request to greetd (length-prefixed JSON)
    async fn send_request(&mut self, request: Request) -> GreetdResult<()> {
        let msg = serde_json::to_vec(&request)
            .map_err(|e| GreetdError::Protocol(format!("Serialize error: {}", e)))?;

        let len = (msg.len() as u32).to_ne_bytes();
        self.stream.write_all(&len).await
            .map_err(|e| GreetdError::ConnectionFailed(format!("Write error: {}", e)))?;
        self.stream.write_all(&msg).await
            .map_err(|e| GreetdError::ConnectionFailed(format!("Write error: {}", e)))?;
        self.stream.flush().await
            .map_err(|e| GreetdError::ConnectionFailed(format!("Flush error: {}", e)))?;

        Ok(())
    }

    /// Read a response from greetd (length-prefixed JSON)
    async fn read_response(&mut self) -> GreetdResult<Response> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await
            .map_err(|e| GreetdError::ConnectionFailed(format!("Read error: {}", e)))?;
        let len = u32::from_ne_bytes(len_buf) as usize;

        if len > 1024 * 1024 {
            return Err(GreetdError::Protocol("Response too large".into()));
        }

        let mut msg_buf = vec![0u8; len];
        self.stream.read_exact(&mut msg_buf).await
            .map_err(|e| GreetdError::ConnectionFailed(format!("Read error: {}", e)))?;

        let response: Response = serde_json::from_slice(&msg_buf)
            .map_err(|e| GreetdError::Protocol(format!("Deserialize error: {}", e)))?;
        Ok(response)
    }
}

/// Convenience function for full authentication flow
pub async fn login(username: &str, password: &str, session: &str) -> GreetdResult<()> {
    let mut client = GreetdClient::connect().await?;
    client.authenticate(username, password).await?;
    client.start_session(session).await?;
    Ok(())
}
