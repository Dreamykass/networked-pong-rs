use crate::input::Input;
use crate::message::*;
use crate::world::*;
use crate::{netw_general, verification};
use ::rand::Rng;
use futures::stream::StreamExt;
use macroquad::prelude::*;

pub async fn server_loop() {
    let mut world = new_world();

    let (world_watch_sender, world_watch_recver) = tokio::sync::watch::channel(world.clone());
    let (input_channel_sender, input_watch_recver) = std::sync::mpsc::channel();

    std::thread::spawn(|| server_netw_thread(world_watch_recver, input_channel_sender));

    loop {
        log::info!("-------- frame begins --------");

        clear_background(BLACK);
        draw_text("SERVER", 20.0, 20.0, 40.0, RED);

        render_world(&world, RED);
        update_world(
            &mut world,
            input_watch_recver.try_recv().unwrap_or_default(),
        );
        world_watch_sender.send(world.clone()).unwrap();

        next_frame().await
    }
}

pub fn update_world(world: &mut World, input: Input) {
    // input
    {
        if input.reset {
            *world = new_world();
            log::info!("world got reset by the client input");
        }

        if input.up {
            world.paddle_left.y -= 0.01;
        }
        if input.down {
            world.paddle_left.y += 0.01;
        }
    }

    // right paddle (computer/bot/ai player)
    {
        if world.ball_pos.y < world.paddle_right.y {
            world.paddle_right.y -= 0.01;
        }
        if world.ball_pos.y > world.paddle_right.y {
            world.paddle_right.y += 0.01;
        }
    }

    // ball
    {
        world.ball_pos += world.ball_vector;

        // bounce off the left-right edges
        if world.ball_pos.x < 0.0 {
            world.ball_pos.x = 0.5;
            world.ball_vector = [0.0, 0.0].into();
        }
        if world.ball_pos.x > 1.0 {
            world.ball_pos.x = 0.5;
            world.ball_vector = [0.0, 0.0].into();
        }

        // bounce off the top/bottom edges
        if world.ball_pos.y < 0.0 {
            world.ball_pos.y = 0.0;
            world.ball_vector.y *= -1.0;
            log::info!("ball bounced off the top edge");
        }
        if world.ball_pos.y > 1.0 {
            world.ball_pos.y = 1.0;
            world.ball_vector.y *= -1.0;
            log::info!("ball bounced off the bottom edge");
        }

        // bounce off the paddles
        if world.ball_pos.x > world.paddle_right.x
            && world.ball_pos.y > world.paddle_right.y - 0.05
            && world.ball_pos.y < world.paddle_right.y + 0.05
        {
            log::info!("ball bounced off the right paddle");
            world.ball_pos.x = world.paddle_right.x;
            world.ball_vector.x *= -1.0;
            world.ball_vector.y = ::rand::thread_rng().gen_range(-0.02..0.02);
        }
        if world.ball_pos.x < world.paddle_left.x
            && world.ball_pos.y > world.paddle_left.y - 0.05
            && world.ball_pos.y < world.paddle_left.y + 0.05
        {
            log::info!("ball bounced off the left paddle");
            world.ball_pos.x = world.paddle_left.x;
            world.ball_vector.x *= -1.0;
            world.ball_vector.y = ::rand::thread_rng().gen_range(-0.02..0.02);
        }
    }
}

#[tokio::main] // converts from async to sync function
async fn server_netw_thread(
    world_watch_recver: tokio::sync::watch::Receiver<World>,
    input_channel_sender: std::sync::mpsc::Sender<Input>,
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
    input_channel_sender: std::sync::mpsc::Sender<Input>,
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
                                input_channel_sender.send(input).unwrap();
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
