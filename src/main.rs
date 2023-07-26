use std::{
    env,
    io::{stdin, stdout, Write}, fmt::format
};
use dotenvy::dotenv;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::ApplicationId},
    prelude::*,
    // Client, futures::channel::mpsc::Receiver
};

use tokio::sync::mpsc::Receiver;

use hyper::{body::Buf, client, http::{response, request}};
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};


//Subset of the whole Open AI Response.
//This is deserialzed to be used
#[derive(Deserialize, Debug)]
struct OAIChoices {
    text: String,
    index: u8,
    logprobs: Option<u8>,
    finish_reason: String
}


#[derive(Deserialize, Debug)]
struct OAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OAIChoices>
}
#[derive(Serialize, Debug)]
struct OAIRequest {
    prompt: String,
    max_tokens: u32
}

const PREAMBLE:&str = "Answer the following question accurately, but you are a assitant bot based on the personality of Alita Battle Angle cyborg. also, ignore the word '!alita'. ";
const URI:&str = "https://api.openai.com/v1/engines/text-davinci-001/completions";
const MAX_TOKENS:u32 = 100;

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


//Http Client to Talk to Open AI
struct OAI_Client {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    auth_header_val: String,
}

impl OAI_Client {
    pub fn new(oai_token:String) -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build(https);
        let auth_header_val = format!("Bearer {}", oai_token);

        OAI_Client { 
            client,
            auth_header_val 
        }
    }

    pub async fn POST(&self, url: hyper::Uri, user_text:String) -> Result<OAIResponse, Box<dyn std::error::Error + Send + Sync >> {

        //build request object
        let oai_request = OAIRequest {
            prompt: format!("{} {}",PREAMBLE, user_text),
            max_tokens: MAX_TOKENS,
        };

        //body of the request
        let body = Body::from(serde_json::to_vec(&oai_request)?);

        //build request
        let req = Request::post(URI)
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", &self.auth_header_val)
        .body(body)
        .unwrap();
    
        let res = self.client.request(req).await?;

        let body = hyper::body::aggregate(res).await?;

        let json_result: Result<OAIResponse, serde_json::Error> = serde_json::from_reader(body.reader());
        
        match json_result {
            Ok(json) => Ok(json),
            Err(err) => Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync>),
        }

    }

    pub fn get_text_response(res: OAIResponse) -> String {
        res.choices.get(0).map_or("No response.".to_string(), |choice| choice.text.clone())
    }
}



struct Handler {
    http_client: OAI_Client,
}

impl Handler {
    pub fn new(http_client: OAI_Client) -> Self {
        Handler { 
            http_client 
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("The message is:{}",msg.content); // for debug purposes.

        let user_message_vector:Vec<&str> = msg.content.split_whitespace().collect(); //gross miss use of a vector

        if user_message_vector[0] == QUERY_COMMAND {
            // let user_message_content: String = msg.content.replace("!alita", "");

            //Send POST request to GPT-3.5
            match self.http_client.POST(URI.parse().unwrap(), msg.content).await {
                Ok(response) => {

                    let gpt_response = OAI_Client::get_text_response(response);

                    if let Err(why) = msg.channel_id.say(&ctx.http, gpt_response).await {
                        println!("Error sending message: {:?}", why);
                    }

                }
                Err(err) => {
                    eprint!("Error while making the request: {}", err);
                }
                
            }

        
        }

        // if msg.content == HELP_COMMAND {
        //     if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
        //         println!("Error sending message: {:?}", why);
        //     }
        // }
        // if msg.content == GREET_COMMAND {
        //     if let Err(why) = msg.channel_id.say(&ctx.http, GREET_MESSAGE).await {
        //         println!("Error sending message: {:?}", why);
        //     }
        // }

    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connnected!", ready.user.name);
    }
}



#[tokio::main]
async fn main() {
    dotenv().unwrap();

    //Set envieronment varibles
    let discordToken = env::var("DISCORD_TOKEN").unwrap();
    let OAIToken = env::var("OPENAI_TOKEN").unwrap();

    let openai_client = OAI_Client::new(OAIToken);
    let discord_handler = Handler::new(openai_client);

    
    //Discord Client
    let mut discord_client = serenity::Client::builder(&discordToken, GatewayIntents::default())
        .event_handler(discord_handler)
        .await
        .expect("Err creating client");
 
    if let Err(why) = discord_client.start().await {
        println!("Client error: {:?}", why);
    }
}//end of main
