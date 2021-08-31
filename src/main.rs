mod client;
mod netw_general;
mod server;
mod verification;
mod world;

#[macroquad::main("Pong")]
async fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}:{}] [{}] : {}",
                chrono::Local::now().format("%H:%M:%S.%3f"),
                record.target(),
                // record.file().unwrap_or("?"),
                record.line().unwrap_or(0),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let env_arg = std::env::args()
        .collect::<Vec<_>>()
        .get(1)
        .expect("The first argument should either be 'client' or 'server', while it was nothing.")
        .clone();

    match env_arg.as_str() {
        "server" => server::server_loop().await,
        "client" => client::client_loop().await,
        _ => {
            panic!(
                "The first argument should either be 'client' or 'server', while it was '{}'.",
                env_arg
            )
        }
    };
}
