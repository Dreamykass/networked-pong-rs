# networked-pong-rs

https://gfycat.com/pl/sparklingkeenaffenpinscher - video of the gameplay

This is a super simple networked pong game,
where the processing is handled by the server, and the client only sends
input and receives position updates. It is single-player - 
the server simulates the second player/bot.

Networking is done with `quinn`/`tokio`. Graphics and input is `macroquad`. And some other crates.

