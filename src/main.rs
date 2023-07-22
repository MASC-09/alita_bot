use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::ApplicationId},
    prelude::*,
    Client
};

const TOKEN_BOT: &str = "MTEzMjA2NDQ0MDk3MjQ5NjkyNg.GRZuxz.zH6aEdAo9rWyfPBEc3bKZfxN_kycfj8X3HJvBI";
const APPLICATION_ID: u64 = 1132075956882911284;

const HELP_MESSAGE: &str = "
Hello there, Human!

You have summoned me. Let's see about getting you what you need.

❓ Need technical help?
➡️ Post in the <#1132075956882911284> channel and other humans will assist you.

❓ Looking for the Code of Conduct?
➡️ Here it is: <https://opensource.facebook.com/code-of-conduct>

❓ Something wrong?
➡️ You can flag an admin with @admin

I hope that resolves your issue!

— HelpBot 🤖
";

const HELP_COMMAND: &str = "!help";
const GREET_COMMAND: &str = "!hello";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("The message is:{}",msg.content);

        if msg.content == HELP_COMMAND {
            if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connnected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // let token = env::var("DISCORD_TOKEN")
    //     .expect("Expected a token in the environment");

    let mut client = Client::builder(&TOKEN_BOT, GatewayIntents::default())
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    // client.set_application_id(ApplicationId(APPLICATION_ID));

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
