use crate::input::Input;
use crate::message::Message;
use crate::world::*;
use crate::{netw_general, verification};
use ::rand::Rng;
use futures::stream::StreamExt;
use macroquad::prelude::*;

pub async fn client_loop() {
    let mut world = new_world();
    let mut input = Input::default();

    let (world_watch_sender, world_watch_recver) = tokio::sync::watch::channel(world.clone());
    let (input_watch_sender, input_watch_recver) = tokio::sync::watch::channel(input.clone());

    std::thread::spawn(|| client_netw_thread(world_watch_sender, input_watch_recver));

    loop {
        log::info!("-------- frame begins --------");

        clear_background(BLACK);
        draw_text("CLIENT", 80.0, 20.0, 40.0, WHITE);

        update_world_if_newer(&mut world, world_watch_recver.borrow().clone());
        render_world(&world, WHITE);

        input.up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
        input.down = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
        input.reset = is_key_down(KeyCode::Space) || is_key_down(KeyCode::Escape);
        input_watch_sender.send(input.clone()).unwrap();

        next_frame().await
    }
}

fn update_world_if_newer(last_world: &mut World, new_world: World) {
    if new_world.tick > last_world.tick {
        *last_world = new_world;
    } else {
        log::warn!(
            "did not update the world from the server: {} vs newer {}",
            last_world.tick,
            new_world.tick
        );
    }
}

#[tokio::main] // converts from async to sync function
async fn client_netw_thread(
    world_watch_sender: tokio::sync::watch::Sender<World>,
    input_watch_recver: tokio::sync::watch::Receiver<Input>,
) {
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

    handle_connection(connection, world_watch_sender, input_watch_recver).await;
}

async fn handle_connection(
    mut connection: quinn::NewConnection,
    world_watch_sender: tokio::sync::watch::Sender<World>,
    mut input_watch_recver: tokio::sync::watch::Receiver<Input>,
) {
    let user_id = ::rand::thread_rng().gen_range(10001..99999).to_string();

    // send the username
    for _ in 0..20 {
        let user_id_bytes =
            bincode::serialize(&[Message::UserIdentifier(user_id.clone())]).unwrap();
        connection
            .connection
            .send_datagram(user_id_bytes.into())
            .unwrap();
    }

    loop {
        tokio::select! {
            // receive datagram from server
            Some(datagram) = connection.datagrams.next() => {
                match datagram {
                    Ok(received_bytes) => {
                        log::info!("received datagram from server: {} bytes", received_bytes.len());
                        let message: Message = bincode::deserialize(&received_bytes).unwrap();
                        match message {
                            Message::WorldUpdate(world) => {
                                log::info!("received a world update");
                                world_watch_sender.send(world).unwrap();
                            }
                            Message::UserIdentifier(_) => panic!(),
                            Message::UserInput(_) => panic!(),
                        }
                    }
                    Err(err) => {
                        log::warn!("datagram read error: {:?}", err);
                        break;
                    }
                }
            }

            // send input to server
            Ok(()) = input_watch_recver.changed() => {
                let input = Message::UserInput(input_watch_recver.borrow().clone());
                let msg = bincode::serialize(&[input]).unwrap();

                match connection.connection.send_datagram(msg.into()) {
                    Ok(()) => {
                        log::info!("sent input update to server");
                    }
                    Err(err) => {
                        log::info!("datagram send error: {:?}", err);
                        break;
                    }
                }
            }
        }
    }
}
