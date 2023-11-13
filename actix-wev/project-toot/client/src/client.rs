use {
	crate::{
		errors::ClientError,
		types::{IncomingMessage, MessageWithId, SendMessageArgs},
	},
	aes::{
		cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
		Aes256Enc,
	},
	borsh::BorshSerialize,
	chrono::offset::Utc,
	secp256k1::{
		ecdh::SharedSecret,
		hashes::sha256,
		KeyPair,
		Message,
		PublicKey,
		Secp256k1,
		SecretKey,
	},
	std::net::TcpStream,
	tungstenite::{connect, stream::MaybeTlsStream, WebSocket},
};

pub struct Client {
	counter: u64,
	secp: Secp256k1<secp256k1::All>,
	kp: KeyPair,
	ws: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl Client {
	pub fn with_secret_key(sk: SecretKey) -> Self {
		let secp = Secp256k1::new();
		let kp = KeyPair::from_secret_key(&secp, &sk);

		Self {
			counter: 0,
			secp,
			kp,
			ws: None,
		}
	}

	pub fn public_key(&self) -> PublicKey {
		self.kp.public_key()
	}

	pub fn connect(&mut self, url: String) -> Result<(), ClientError> {
		if self.ws.is_some() {
			return ClientError::AlreadyConnected.into()
		}

		match connect(url) {
			Ok((ws, _)) => {
				self.ws = Some(ws);
				Ok(())
			},
			Err(err) => ClientError::WebSocketError(err).into(),
		}
	}

	pub fn send_message(
		&mut self,
		receiver: PublicKey,
		save_send_date: bool,
		raw: Vec<u8>,
	) -> Result<(), ClientError> {
		if self.ws.is_none() {
			return ClientError::NotConnected.into()
		}

		let id = self.counter;
		self.counter += 1;

		let shared = SharedSecret::new(&receiver, &self.kp.secret_key());
		let timestamp = Utc::now().timestamp();

		let mut blocks = raw
			.chunks(16)
			.map(|chunk| *GenericArray::from_slice(chunk))
			.collect::<Vec<_>>();

		Aes256Enc::new(&GenericArray::from(shared.secret_bytes())).encrypt_blocks(&mut blocks);

		let mut data = Vec::<u8>::new();
		for chunk in blocks {
			data.extend_from_slice(chunk.as_slice());
		}

		let (sender_x, sender_p) = self.kp.x_only_public_key();
		let (receiver_x, receiver_p) = receiver.x_only_public_key();

		let res = self.ws.as_mut().unwrap().send(
			MessageWithId {
				id,
				message: IncomingMessage::SendMessage(SendMessageArgs {
					sender_x: sender_x.serialize(),
					sender_p: sender_p as u8,
					receiver_x: receiver_x.serialize(),
					receiver_p: receiver_p as u8,
					signature: self
						.secp
						.sign_ecdsa(
							&Message::from_hashed_data::<sha256::Hash>(
								&[&timestamp.to_le_bytes(), data.as_slice()].concat(),
							),
							&self.kp.secret_key(),
						)
						.serialize_compact(),
					timestamp,
					save_send_date,
					data,
				}),
			}
			.try_to_vec()
			.unwrap()
			.into(),
		);

		if let Some(err) = res.err() {
			return ClientError::WebSocketError(err).into()
		}

		Ok(())
	}
}

impl From<KeyPair> for Client {
	fn from(kp: KeyPair) -> Self {
		Self {
			counter: 0,
			secp: Secp256k1::new(),
			kp,
			ws: None,
		}
	}
}
