# networked-pong-rs

https://gfycat.com/pl/sparklingkeenaffenpinscher - video of the gameplay

This is a super simple, networked pong game,
where the processing is handled by the server, and the client only sends
input and receives position updates. It is single-player - 
the server simulates the second player/bot.

`cargo run -- server` to run the server  
`cargo run -- client` to run the client  
On the client, UP/W key moves the left paddle up,
DOWN/S key moves the left paddle down,
and SPACE/ESC resets the game.

Networking is done with `quinn`/`tokio`. Graphics and input is `macroquad`.
And there's some other crates used for other things too.

The game should probably run at around 60 FPS. *It works on my machine.* 

# networking stuff

To mitigate packet reordering, the server sends its current tick number
along with the world state update, and the client updates its world only if the
world state update is newer than the last.

~~To mitigate packet drops, the client sends the last 10 input states,
along with the current tick, and the server .~~

