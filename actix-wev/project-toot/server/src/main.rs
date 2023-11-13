pub mod constants;
pub mod session;
pub mod types;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let manager = bb8_postgres::PostgresConnectionManager::new(
		tokio_postgres::Config::new()
			.user("project-toot")
			.password("toot-tcejorp")
			.clone(),
		tokio_postgres::NoTls,
	);
	let pool = bb8::Pool::builder()
		.max_size(32)
		.build(manager)
		.await
		.expect("Couldn't connect to postgres due error");

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.service(session::ws_connect)
			.app_data(actix_web::web::Data::new(pool.clone()))
	})
	.bind(("127.0.0.1", 8080))?
	.run()
	.await
}
