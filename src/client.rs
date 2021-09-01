use crate::world::*;
use crate::{netw_general, verification};
use macroquad::prelude::*;

pub async fn client_loop() {
    let world = new_world();

    std::thread::spawn(client_netw_thread);

    loop {
        log::info!("-------- frame begins --------");

        clear_background(BLACK);
        draw_text("CLIENT", 20.0, 20.0, 40.0, WHITE);

        render_world(&world);

        next_frame().await
    }
}

#[tokio::main] // converts from async to sync function
async fn client_netw_thread() {
    log::info!("client_netw_thread");

    let mut endpoint_builder = quinn::Endpoint::builder();
    endpoint_builder.default_client_config(verification::new_insecure_client_config());
    let (endpoint, _) = endpoint_builder.bind(&netw_general::client_addr()).unwrap();

    let connection = endpoint
        .connect(&netw_general::server_addr(), netw_general::SERVER_NAME)
        .unwrap()
        .await
        .unwrap();
    log::info!("connection ok");

    // connection.send_datagram(b"test".into()).await.unwrap();
    // connection
    //     .connection
    //     .send_datagram("Hello from the client!".into());
}
