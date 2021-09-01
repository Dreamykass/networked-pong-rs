use crate::input::Input;
use crate::message::*;
use crate::world::*;
use crate::{netw_general, verification};
use futures::stream::StreamExt;
use macroquad::prelude::*;

pub async fn server_loop() {
    let mut world = new_world();

    let (world_watch_sender, world_watch_recver) = tokio::sync::watch::channel(world.clone());
    let (input_channel_sender, mut input_watch_recver) = tokio::sync::mpsc::channel(1);

    std::thread::spawn(|| server_netw_thread(world_watch_recver, input_channel_sender));

    loop {
        log::info!("-------- frame begins --------");

        clear_background(BLACK);
        draw_text("SERVER", 20.0, 20.0, 40.0, RED);

        render_world(&world, RED);
        update_world(&mut world, input_watch_recver.recv().await.unwrap());
        world_watch_sender.send(world.clone()).unwrap();

        next_frame().await
    }
}

pub fn update_world(world: &mut World, input: Input) {
    world.ball_pos += world.ball_vector;
    if world.ball_pos.x < 0.0 {
        world.ball_pos.x = 0.5;
    }
    if world.ball_pos.x > 1.0 {
        world.ball_pos.x = 0.5;
    }
    if world.ball_pos.y < 0.0 {
        world.ball_pos.y = 0.5;
    }
    if world.ball_pos.y > 1.0 {
        world.ball_pos.y = 0.5;
    }

    if input.reset {
        *world = new_world();
    }
}

#[tokio::main] // converts from async to sync function
async fn server_netw_thread(
    world_watch_recver: tokio::sync::watch::Receiver<World>,
    input_channel_sender: tokio::sync::mpsc::Sender<Input>,
) {
    log::info!("server_netw_thread");

    let mut endpoint_builder = quinn::Endpoint::builder();

    let mut server_config_builder = quinn::ServerConfigBuilder::default();
    let (cert, key) = verification::generate_self_signed_cert().unwrap();
    server_config_builder
        .certificate(quinn::CertificateChain::from_certs(vec![cert]), key)
        .unwrap();

    endpoint_builder.listen(server_config_builder.build());

    let (_endpoint, mut incoming) = endpoint_builder.bind(&netw_general::server_addr()).unwrap();

    while let Some(connecting) = incoming.next().await {
        let connection: quinn::NewConnection = connecting.await.unwrap();
        log::info!("received a new connection");
        let world_watch_recver = world_watch_recver.clone();
        tokio::spawn(handle_connection(
            connection,
            world_watch_recver,
            input_channel_sender.clone(),
        ));
    }
}

async fn handle_connection(
    mut connection: quinn::NewConnection,
    mut world_watch_recver: tokio::sync::watch::Receiver<World>,
    mut input_channel_sender: tokio::sync::mpsc::Sender<Input>,
) {
    let mut user_id = "unknown".to_string();

    loop {
        tokio::select! {
            // send world update by datagram to client
            Ok(()) = world_watch_recver.changed() => {
                let world = world_watch_recver.borrow().clone();
                let msg = bincode::serialize(&[Message::WorldUpdate(world)]).unwrap();

                match connection.connection.send_datagram(msg.into()){
                    Ok(()) => {
                        log::info!("sent world update to {}", user_id);
                    }
                    Err(err) => {
                        log::info!("({}) datagram send error: {:?}", user_id, err);
                        break;
                    }
                }
            }

            // receive datagram from client
            Some(datagram) = connection.datagrams.next() => {
                match datagram {
                    Ok(received_bytes) => {
                        log::info!("received datagram from {}: {} bytes", user_id, received_bytes.len());
                        let message: Message = bincode::deserialize(&received_bytes).unwrap();

                        match message {
                            Message::WorldUpdate(_) => panic!(),
                            Message::UserIdentifier(new_user_id) => {
                                user_id = new_user_id;
                            },
                            Message::UserInput(input) => {
                                log::info!("{:?}", input);
                                input_channel_sender.try_send(input);
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!("({}) datagram read error: {:?}", user_id, err);
                        break;
                    }
                }

            }
        }
    }

    log::warn!("connection closed/over");
}
