// ~/hypr-greeter/src/greetd_client.rs
// greetd IPC client implementation

use greetd_ipc::{AuthMessageType, ErrorType, Request, Response};
use std::error::Error;
use tokio::net::UnixStream;

/// Result type for greetd operations
pub type GreetdResult<T> = Result<T, Box<dyn Error>>;

/// greetd client for authentication
pub struct GreetdClient {
    /// Unix socket connection to greetd
    stream: UnixStream,
}

impl GreetdClient {
    /// Connect to greetd daemon
    pub async fn connect() -> GreetdResult<Self> {
        let stream = UnixStream::connect("/run/greetd.sock").await?;
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
        self.send_request(&request).await?;
        
        // Handle response
        match self.read_response().await? {
            Response::AuthMessage { auth_message_type, .. } => {
                match auth_message_type {
                    AuthMessageType::Secret | AuthMessageType::SecretVisible => {
                        // Send password
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
    async fn send_password(&mut self, password: &str) -> GreetdResult<()> {
        let request = Request::PostAuthMessageResponse {
            response: Some(password.to_string()),
        };
        self.send_request(&request).await?;
        
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
        
        self.send_request(&request).await?;
        
        // Session should start, we might not get a response
        // as greetd might exec into the session
        Ok(())
    }
    
    /// Cancel the current session
    pub async fn cancel_session(&mut self) -> GreetdResult<()> {
        let request = Request::CancelSession;
        self.send_request(&request).await?;
        
        match self.read_response().await? {
            Response::Success => Ok(()),
            Response::Error { error_type, description } => {
                Err(format!("Cancel failed: {:?} - {}", error_type, description).into())
            }
            _ => Err("Unexpected response to cancel".into()),
        }
    }
    
    /// Send a request to greetd
    async fn send_request(&mut self, request: &Request) -> GreetdResult<()> {
        greetd_ipc::send(&mut self.stream, request).await?;
        Ok(())
    }
    
    /// Read a response from greetd
    async fn read_response(&mut self) -> GreetdResult<Response> {
        let response = greetd_ipc::read(&mut self.stream).await?;
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