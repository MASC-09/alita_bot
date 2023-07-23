use std::{
    env,
    io::{stdin, stdout, Write}
};
use dotenvy::dotenv;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::ApplicationId},
    prelude::*,
    // Client, futures::channel::mpsc::Receiver
};
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole, ChatCompletionDelta},
    set_key,
};
use tokio::sync::mpsc::Receiver;

const APPLICATION_ID: u64 = 1132075956882911284;

const HELP_MESSAGE: &str = "
Hello there, Human!

You have summoned me. Let's see about getting you what you need.

❓ Need technical help?
➡️ Post in the <#1132075956882911284> channel and other humans will assist you.

❓ Something wrong?
➡️ You can flag an admin with @admin

I hope that resolves your issue!

— Alita 🤖
";

const GREET_MESSAGE: &str = "HELLO USER WELCOME";

const HELP_COMMAND: &str = "!help";
const GREET_COMMAND: &str = "!hello";
const QUERY_COMMAND: &str = "!alita";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("The message is:{}",msg.content);
        // if msg.content == ""{
        //     println!("Message is empty!");
        // };
        let user_message_vector:Vec<&str> = msg.content.split_whitespace().collect(); //gross miss use of a vector

        if user_message_vector[0] == QUERY_COMMAND {
            let user_message_content: String = msg.content.replace("!alita", "");

            let mut messages = vec![ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some("You are a large language model built into a Discord Bot called Alita".to_string()),
                name: None,
                function_call: None,
            }];

            messages.push(ChatCompletionMessage { 
                role: ChatCompletionMessageRole::User,
                content: Some(user_message_content), 
                name: None, 
                function_call: None 
            });

            let chat_stream = ChatCompletionDelta::builder("gpt-3.5-turbo", messages.clone())
                .create_stream()
                .await
                .unwrap();
            
            let chat_completion: ChatCompletion = listen_for_tokens(chat_stream).await;
            let returned_message = chat_completion.choices.first().unwrap().message.content.clone().unwrap().trim();

            if let Err(why) = msg.channel_id.say(&ctx.http, returned_message).await {
                println!("Error sending message: {:?}", why);
            }
        }

        if msg.content == HELP_COMMAND {
            if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg.content == GREET_COMMAND {
            if let Err(why) = msg.channel_id.say(&ctx.http, GREET_MESSAGE).await {
                println!("Error sending message: {:?}", why);
            }
        }

    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connnected!", ready.user.name);
    }
}

async fn listen_for_tokens(mut chat_stream: Receiver<ChatCompletionDelta>) -> ChatCompletion {
    let mut merged: Option<ChatCompletionDelta> = None;
    while let Some(delta) = chat_stream.recv().await {
        let choice = &delta.choices[0];
        if let Some(role) = &choice.delta.role {
            print!("{:#?}: ", role);
        }
        if let Some(content) = &choice.delta.content {
            print!("{}", content);
        }
        if let Some(_) = &choice.finish_reason {
            // The message being streamed has been fully received.
            print!("\n");
        }
        stdout().flush().unwrap();
        // Merge completion into accrued.
        match merged.as_mut() {
            Some(c) => {
                c.merge(delta).unwrap();
            }
            None => merged = Some(delta),
        };
    }
    merged.unwrap().into()
}


#[tokio::main]
async fn main() {
    dotenv().unwrap();

    //Set envieronment varibles
    let token = env::var("DISCORD_TOKEN").unwrap();
    set_key(env::var("OPENAI_KEY").unwrap());

    let mut client = Client::builder(&token, GatewayIntents::default())
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    // client.set_application_id(ApplicationId(APPLICATION_ID));

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}//end of main
