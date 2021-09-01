use crate::world::*;
use crate::{netw_general, verification};
use futures::stream::StreamExt;
use macroquad::prelude::*;

pub async fn server_loop() {
    let mut world = new_world();

    std::thread::spawn(server_netw_thread);

    loop {
        log::info!("-------- frame begins --------");

        clear_background(BLACK);
        draw_text("SERVER", 20.0, 20.0, 40.0, RED);

        render_world(&world);
        update_world(&mut world);

        next_frame().await
    }
}

pub fn update_world(world: &mut World) {
    world.ball_pos += world.ball_vector;
}

#[tokio::main] // converts from async to sync function
async fn server_netw_thread() {
    log::info!("server_netw_thread");

    let mut endpoint_builder = quinn::Endpoint::builder();

    let mut server_config_builder = quinn::ServerConfigBuilder::default();
    let (cert, key) = verification::generate_self_signed_cert().unwrap();
    server_config_builder
        .certificate(quinn::CertificateChain::from_certs(vec![cert]), key)
        .unwrap();

    endpoint_builder.listen(server_config_builder.build());

    let (endpoint, mut incoming) = endpoint_builder.bind(&netw_general::server_addr()).unwrap();

    while let Some(conn) = incoming.next().await {
        let mut connection: quinn::NewConnection = conn.await.unwrap();
        log::info!("received a connection");

        // while let Some(Ok(received_bytes)) = connection.datagrams.next().await {
        //     log::info!("received datagram: {:?}", received_bytes);
        // }

        // Save connection somewhere, start transferring, receiving data, see DataTransfer tutorial.
    }
}
