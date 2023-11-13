pub enum ClientError {
	WebSocketError(tungstenite::Error),
	AlreadyConnected,
	NotConnected,
}

impl<T> From<ClientError> for Result<T, ClientError> {
	fn from(value: ClientError) -> Self {
		Self::Err(value)
	}
}
