use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct MessageWithId<T: BorshSerialize + BorshDeserialize> {
	pub id: u64,
	pub message: T,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct SendMessageArgs {
	pub sender_x: [u8; 32],
	pub sender_p: u8,
	pub receiver_x: [u8; 32],
	pub receiver_p: u8,
	pub signature: [u8; 64],
	pub timestamp: i64,
	pub save_timestamp: bool,
	pub data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum IncomingMessage {
	SendMessage(SendMessageArgs),
	ListMessages,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum OutcomingMessage {
	Success,
	Error(String),
}
