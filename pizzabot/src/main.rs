//! This example shows a clone of the original PIZZABOT.
//! 
//! To implement PIZZABOT, we create a websocket
//! connection to the airma.sh server (us-s1, and FFA1).
//! After logging into the websocket server, we then
//! listen in on all public chat messages and respond
//! when one of our commands is said on public chat.
//! 
//! # Things not implemented here
//! 
//! - Rejoin after being disconnected.
//! - Chat throttling.
//! - Avoiding being disconnected.
//! - Attempt to spectate on join.
//! - Any error handling at all.
//! 

extern crate airmash_protocol;
extern crate websocket;

use airmash_protocol::{
    ClientPacket,
    ServerPacket,
    to_bytes,
    from_bytes
};
use airmash_protocol::client::{Login, Chat, Whisper, Pong};
use airmash_protocol::server::Ping;

use websocket::ClientBuilder;
use websocket::stream::Stream;
use websocket::sync::client::Client;
use websocket::message::OwnedMessage;

const BOT_PING: &'static str = "-bot-ping";
const GET_PIZZA: &'static str = "-get-pizza";
const NAME: &'static str = "PIZZABOT";

const MESSAGE: &'static str = "Order airmash pizza here: http://tiny.cc/airmash-pizza . Use code 'Detect' for discounts.";
const BOT_MSG: &'static str = "I am PIZZABOT. Owner: Dominos";

// Connect PIZZABOT to FFA1 in the US region. 
const URL: &'static str = "wss://game-us-s1.airma.sh/ffa1";

fn send_login<S>(ws: &mut Client<S>)
    where S: Stream
{
    // Build our login packet
    let login = Login {
        // This must always be 5, see the 
        // airmash_protocol docs.
        protocol: 5,
        // Set our name
        name: NAME.to_string(),
        // We have no session token
        session: "none".to_string(),
        // Doesn't really matter what we set these to
        horizon_x: 1000,
        horizon_y: 1000,
        // Use the UN flag
        flag: "XX".to_string()
    };

    // Serialize the login packet
    let bytes = to_bytes(&ClientPacket::Login(login)).unwrap();

    ws.send_message(&OwnedMessage::Binary(bytes)).unwrap();
}
// We need to respond to pings or the server
// will disconnect us.
fn send_pong<S>(ws: &mut Client<S>, ping: &Ping) 
    where S: Stream
{
    let pong = Pong {
        num: ping.num
    };

    let bytes = to_bytes(&ClientPacket::Pong(pong)).unwrap();
    ws.send_message(&OwnedMessage::Binary(bytes)).unwrap();
}
// Respond to -bot-ping
fn send_bot_ping_response<S>(ws: &mut Client<S>, player: u16)
    where S: Stream
{
    // Whisper our -bot-ping response back to the player
    let response = Whisper {
        id: player,
        text: BOT_MSG.to_string()
    };

    let bytes = to_bytes(&ClientPacket::Whisper(response)).unwrap();
    ws.send_message(&OwnedMessage::Binary(bytes)).unwrap();
}
// Respond to -get-pizza
fn send_get_pizza_response<S>(ws: &mut Client<S>) 
    where S: Stream
{
    let response = Chat {
        text: MESSAGE.to_string()
    };

    let bytes = to_bytes(&ClientPacket::Chat(response)).unwrap();
    ws.send_message(&OwnedMessage::Binary(bytes)).unwrap();
}

fn main() {
    let mut ws = ClientBuilder::new(URL)
        .unwrap()
        .connect_secure(None)
        .unwrap();

    // Login to the server here
    send_login(&mut ws);

    while let Ok(msg) = ws.recv_message() {
        // Check to see if the server closed the connection
        if msg.is_close() {
            break;
        }
        
        // Don't handle non-data websocket messages
        assert!(msg.is_data());

        // Only handle binary messages within this example
        // the server shouldn't be sending any other messages
        let bytes = match msg {
            OwnedMessage::Binary(bytes) => bytes,
            _ => unimplemented!()
        };

        let tmp = from_bytes::<ServerPacket>(&bytes[..]);
        if tmp.is_err() {
            let _unused = 5;
        }

        // Actually decode our packet from the packet bytes
        let packet = tmp.unwrap();

        match packet {
            ServerPacket::Ping(ping) => send_pong(&mut ws, &ping),
            ServerPacket::ChatPublic(chat) => {
                // Here, we check to see if the message is
                // one of the ones we want to respond to.
                // We don't handle packets that aren't 
                // all lowercase here, a full-fledged bot
                // would probably want to do this.
                if chat.text == BOT_PING {
                    send_bot_ping_response(&mut ws, chat.id);
                }
                else if chat.text == GET_PIZZA {
                    send_get_pizza_response(&mut ws);
                }
            },
            _ => ()
        }
    }
}
