// ~/hypr-greeter/src/greetd_client.rs
// greetd IPC client implementation

use greetd_ipc::{AuthMessageType, Request, Response};
use std::error::Error;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Result type for greetd operations
pub type GreetdResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// greetd client for authentication
pub struct GreetdClient {
    /// Unix socket connection to greetd
    stream: UnixStream,
}

impl GreetdClient {
    /// Connect to greetd daemon
    pub async fn connect() -> GreetdResult<Self> {
        // First check if GREETD_SOCK environment variable is set
        let socket_path = std::env::var("GREETD_SOCK")
            .unwrap_or_else(|_| "/run/greetd.sock".to_string());
            
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream })
    }
    
    /// Authenticate a user with password
    pub async fn authenticate(
        &mut self,
        username: &str,
        password: &str,
    ) -> GreetdResult<()> {
        // Create session for user
        let request = Request::CreateSession {
            username: username.to_string(),
        };
        self.send_request(request).await?;
        
        // Handle response
        match self.read_response().await? {
            Response::AuthMessage { auth_message_type, .. } => {
                match auth_message_type {
                    AuthMessageType::Secret { .. } => {
                        // Send password exactly as provided, do not trim or alter whitespace
                        self.send_password(password).await?;
                    }
                    _ => return Err("Unexpected auth message type".into()),
                }
            }
            Response::Error { error_type, description } => {
                return Err(format!("Auth error: {:?} - {}", error_type, description).into());
            }
            _ => return Err("Unexpected response during auth".into()),
        }
        
        Ok(())
    }

    /// Send password response
    /// Password is sent verbatim, including any whitespace.
    async fn send_password(&mut self, password: &str) -> GreetdResult<()> {
        let request = Request::PostAuthMessageResponse {
            // Do NOT trim or modify the password; send as-is
            response: Some(password.to_string()),
        };
        self.send_request(request).await?;
        
        // Check if authentication succeeded
        match self.read_response().await? {
            Response::Success => Ok(()),
            Response::Error { error_type, description } => {
                Err(format!("Authentication failed: {:?} - {}", error_type, description).into())
            }
            _ => Err("Unexpected response after password".into()),
        }
    }
    
    /// Start a session with the specified command
    pub async fn start_session(&mut self, command: &str) -> GreetdResult<()> {
        // Parse command into arguments
        let cmd_parts: Vec<String> = command
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        if cmd_parts.is_empty() {
            return Err("Empty session command".into());
        }
        
        let request = Request::StartSession {
            cmd: cmd_parts,
            env: vec![], // Environment will be set up by greetd
        };
        
        self.send_request(request).await?;
        
        // Session should start, we might not get a response
        // as greetd might exec into the session
        Ok(())
    }
    
    /// Cancel the current session
    pub async fn cancel_session(&mut self) -> GreetdResult<()> {
        let request = Request::CancelSession;
        self.send_request(request).await?;
        match self.read_response().await? {
            Response::Success => Ok(()),
            Response::Error { error_type, description } => {
                Err(format!("Cancel failed: {:?} - {}", error_type, description).into())
            }
            _ => Err("Unexpected response to cancel".into()),
        }
    }
    
    /// Send a request to greetd
    /// The greetd IPC protocol uses length-prefixed JSON messages
    async fn send_request(&mut self, request: Request) -> GreetdResult<()> {
        // Serialize the request to JSON
        let msg = serde_json::to_vec(&request)?;
        
        // Write length prefix (4 bytes, native endian)
        let len = (msg.len() as u32).to_ne_bytes();
        self.stream.write_all(&len).await?;
        
        // Write the JSON message
        self.stream.write_all(&msg).await?;
        self.stream.flush().await?;
        
        Ok(())
    }
    
    /// Read a response from greetd
    /// The greetd IPC protocol uses length-prefixed JSON messages
    async fn read_response(&mut self) -> GreetdResult<Response> {
        // Read length prefix (4 bytes, native endian)
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_ne_bytes(len_buf) as usize;
        
        // Sanity check to prevent huge allocations
        if len > 1024 * 1024 {
            return Err("Response too large".into());
        }
        
        // Read the JSON message
        let mut msg_buf = vec![0u8; len];
        self.stream.read_exact(&mut msg_buf).await?;
        
        // Deserialize the response
        let response: Response = serde_json::from_slice(&msg_buf)?;
        Ok(response)
    }
}

/// Convenience function for full authentication flow
pub async fn login(username: &str, password: &str, session: &str) -> GreetdResult<()> {
    let mut client = GreetdClient::connect().await?;
    
    // Authenticate
    client.authenticate(username, password).await?;
    
    // Start session
    client.start_session(session).await?;
    
    Ok(())
}