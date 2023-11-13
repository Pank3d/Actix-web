use {
	crate::{
		constants::MAXIMUM_SEND_MESSAGE_TIMESTAMP_OFFSET,
		types::{IncomingMessage, MessageWithId, OutcomingMessage, SendMessageArgs},
	},
	actix_web::web,
	actix_ws as ws,
	borsh::{BorshDeserialize, BorshSerialize},
	chrono::{DateTime, NaiveDateTime, Utc},
	futures::StreamExt,
	secp256k1::{
		ecdsa::Signature,
		hashes::sha256,
		Message,
		Parity,
		PublicKey,
		Secp256k1,
		XOnlyPublicKey,
	},
};

pub type DBPool = bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>;

async fn handle_send_message(
	pool: web::Data<DBPool>,
	session: &mut ws::Session,
	id: u64,
	args: SendMessageArgs,
) -> Result<(), ws::Closed> {
	let SendMessageArgs {
		sender_x,
		sender_p,
		receiver_x,
		receiver_p,
		signature,
		timestamp,
		save_timestamp,
		data,
	} = args;

	let datetime = DateTime::<Utc>::from_naive_utc_and_offset(
		NaiveDateTime::from_timestamp_millis(timestamp).unwrap(),
		Utc,
	);

	{
		let now = Utc::now();
		if datetime < (now - MAXIMUM_SEND_MESSAGE_TIMESTAMP_OFFSET) ||
			datetime > (now + MAXIMUM_SEND_MESSAGE_TIMESTAMP_OFFSET)
		{
			return session
				.binary(
					MessageWithId {
						id,
						message: OutcomingMessage::Error(
							"Maximum interval from sent timestamp and current one is 30s"
								.to_string(),
						),
					}
					.try_to_vec()
					.unwrap(),
				)
				.await
		}
	}

	let sender = PublicKey::from_x_only_public_key(
		XOnlyPublicKey::from_slice(&sender_x).unwrap(),
		match sender_p {
			0 => Parity::Even,
			1 => Parity::Odd,
			_ => {
				return session
					.binary(
						MessageWithId {
							id,
							message: OutcomingMessage::Error("Invalid sender_p value".to_string()),
						}
						.try_to_vec()
						.unwrap(),
					)
					.await
			},
		},
	);

	// let receiver = // to check if receiver is correct
	PublicKey::from_x_only_public_key(
		XOnlyPublicKey::from_slice(&receiver_x).unwrap(),
		match receiver_p {
			0 => Parity::Even,
			1 => Parity::Odd,
			_ => {
				return session
					.binary(
						MessageWithId {
							id,
							message: OutcomingMessage::Error(
								"Invalid receiver_p value".to_string(),
							),
						}
						.try_to_vec()
						.unwrap(),
					)
					.await
			},
		},
	);

	let signature = Signature::from_compact(&signature).unwrap();

	let conn = pool.get().await.unwrap();
	let (last_signature, is_first) = if let Some(row) = conn
		.query_opt(
			"SELECT (\"signature\",) FROM \"last_signatures\" WHERE \"pubkey\"=$1",
			&[&sender.serialize().to_vec()],
		)
		.await
		.unwrap()
	{
		let signature_raw = row.get::<_, Vec<u8>>(0);
		if signature_raw.len() == 64 {
			(
				TryInto::<[u8; 64]>::try_into(signature_raw.as_ref()).unwrap(),
				false,
			)
		} else {
			([0; 64], false)
		}
	} else {
		([0; 64], true)
	};

	let message = Message::from_hashed_data::<sha256::Hash>(
		&[&last_signature[..], &timestamp.to_le_bytes()[..], &data].concat(),
	);

	if !Secp256k1::verification_only()
		.verify_ecdsa(&message, &signature, &sender)
		.is_ok()
	{
		return session
			.binary(
				MessageWithId {
					id,
					message: OutcomingMessage::Error("Unsigned data".to_string()),
				}
				.try_to_vec()
				.unwrap(),
			)
			.await
	}

	conn.execute(
		if is_first {
			"INSERT INTO \"last_signatures\" VALUES ($1, $2);"
		} else {
			"UPDATE \"last_signatures\" SET \"signature\"=$2 WHERE \"pubkey\"=$1;"
		},
		&[
			&sender.serialize().to_vec(),
			&signature.serialize_compact().to_vec(),
		],
	)
	.await
	.unwrap();

	conn.execute(
		"INSERT INTO \"messages\" VALUES ($1, $2, $3, $4, $5, $6);",
		&[
			&sender_x.to_vec(),
			&(sender_p as i8),
			&receiver_x.to_vec(),
			&(receiver_p as i8),
			&if save_timestamp {
				datetime.naive_utc()
			} else {
				DateTime::UNIX_EPOCH.naive_utc()
			},
			&data,
		],
	)
	.await
	.unwrap();

	session
		.binary(
			MessageWithId {
				id,
				message: OutcomingMessage::Success,
			}
			.try_to_vec()
			.unwrap(),
		)
		.await
}

async fn handle_session(
	pool: web::Data<DBPool>,
	mut session: ws::Session,
	mut stream: ws::MessageStream,
) -> Result<(), ws::Closed> {
	let reason = loop {
		match stream.next().await {
			Some(Ok(ws::Message::Binary(bytes))) => {
				match MessageWithId::<IncomingMessage>::deserialize(&mut &bytes[..]) {
					Ok(MessageWithId { id, message }) => match message {
						IncomingMessage::SendMessage(args) => {
							handle_send_message(pool.clone(), &mut session, id, args).await?
						},
						IncomingMessage::ListMessages => {},
					},
					Err(_) => {},
				}
			},
			Some(Ok(ws::Message::Ping(_))) => session.pong(&[]).await?,
			Some(Ok(_)) => {},
			_ => break None,
		};
	};

	session.close(reason).await
}

#[actix_web::get("/ws")]
pub async fn ws_connect(
	pool: web::Data<DBPool>,
	req: actix_web::HttpRequest,
	body: web::Payload,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
	let (res, session, stream) = actix_ws::handle(&req, body)?;
	actix_web::rt::spawn(handle_session(pool, session, stream));
	Ok(res)
}
