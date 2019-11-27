extern crate reqwest;

use clap::{crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use log::{info, LevelFilter};
use num_format::{Locale, ToFormattedString};
use std::time::Instant;
use std::{fs, u64};
use std::error::Error;
use reqwest::Client;
use serde::{Deserialize};

fn main() {
    let now = Instant::now();

    let types = ["public_channel", "private_channel", "mpim", "im"];

    let options = App::new("Tidy Slack")
        .version(crate_version!())
        .author("Brandon Frohs <brandon@19.codes>")
        .about("Deletes messages from slack.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        // Subcommands
        .subcommand(SubCommand::with_name("ls")
            .about("List conversations the authenticated user can access.")
            .arg(
                Arg::with_name("exclude_archived")
                    .short("e")
                    .long("exclude_archived")
                    .help("If provided, archived channels will be excluded.")
            )
            .arg(
                Arg::with_name("types")
                    .long("types")
                    .possible_values(&types)
                    .takes_value(true)
                    .multiple(true)
                    .help("Types of conversations to list.")
            )
        )
        // Verbosity level
        .arg(
            Arg::with_name("silent")
                .short("s")
                .long("silent")
                .conflicts_with_all(&["quiet", "verbose"])
                .help("Silences all output."),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .multiple(true)
                .long("quiet")
                .conflicts_with("verbose")
                .help("Shows less detail. -q shows less detail, -qq shows least detail, -qqq is equivalent to --silent."),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .long("verbose")
                .help("Shows more detail. -v shows more detail, -vv shows most detail."),
        )
        .get_matches();

    // Default log level is Info
    // --silent switches level to Off
    // -v, --verbose increases level to debug
    // -vv increases level to Trace
    // -q, --quiet decreases level to Error
    // -qq decreases level to Off
    let filter: LevelFilter = if options.is_present("silent") {
        LevelFilter::Off
    } else {
        match options.occurrences_of("verbose") {
            0 => match options.occurrences_of("quiet") {
                3..=u64::MAX => LevelFilter::Off,
                2 => LevelFilter::Error,
                1 => LevelFilter::Warn,
                0 => LevelFilter::Info, // Default
            },
            1 => LevelFilter::Debug,
            2..=u64::MAX => LevelFilter::Trace,
        }
    };

    env_logger::Builder::from_default_env()
        .filter(Some(module_path!()), filter)
        .init();

    if let Some(cmd) = options.subcommand_name() {
        let sub_options = options.subcommand_matches(cmd);
        match cmd {
            "ls" => ls(types, sub_options),
            _ => panic!("Unsupported command: {}", cmd),
        }
    };

    info!(
        "Command completed in {}.{}s",
        now.elapsed().as_secs().to_formatted_string(&Locale::en),
        now.elapsed().subsec_millis()
    );
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ConversationsKind {
    Conversations(Conversations),
    Error(ConversationsError)
}

#[derive(Deserialize, Debug)]
struct ConversationsError {
    ok: bool,
    error: String,
}
impl std::fmt::Display for ConversationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
impl Error for ConversationsError {
}

#[derive(Deserialize, Debug)]
struct Conversations {
    ok: bool,
    warning: Option<String>,
    channels: Vec<Conversation>,
    response_metadata: Metadata,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Conversation {
    PublicChannel(PublicChannel),
    PrivateChannel(PrivateChannel),
    Im(Im),
}

#[derive(Deserialize, Debug)]
struct PublicChannel {
    id: String,
    name: String,
    is_channel: bool,
    is_group: bool,
    is_im: bool,
    created: u64,
    is_archived: bool,
    is_general: bool,
    unlinked: u64,
    name_normalized: String,
    // is_read_only: bool,
    is_shared: bool,
    parent_conversation: Option<String>,
    creator: String,
    is_ext_shared: bool,
    is_org_shared: bool,
    shared_team_ids: Vec<String>, // Not in documentation, but shows up in results
    pending_shared: Vec<String>, // I believe this should always be an empty array?
    pending_connected_team_ids: Vec<String>, // Not in documentation, but shows up in results
    is_pending_ext_shared: bool,
    is_member: bool,
    is_private: bool,
    is_mpim: bool,
    last_read: Option<String>,
    is_open: Option<bool>,
    topic: Topic,
    purpose: Purpose,
    previous_names: Vec<String>,
    num_members: u64,
    priority: Option<u64>,
    // locale: String
}

#[derive(Deserialize, Debug)]
struct PrivateChannel {
    id: String,
    name: String,
    is_channel: bool,
    is_group: bool,
    is_im: bool,
    created: u64,
    is_archived: bool,
    is_general: bool,
    unlinked: u64,
    name_normalized: String,
    is_read_only: Option<bool>, // I'm not seeing this in the response, but it's in documentation, so I made it optional
    is_shared: bool,
    parent_conversation: Option<String>,
    creator: String,
    is_ext_shared: bool,
    is_org_shared: bool,
    shared_team_ids: Vec<String>, // Not in documentation, but shows up in results
    pending_shared: Vec<String>, // I believe this should always be an empty array?
    pending_connected_team_ids: Vec<String>, // Not in documentation, but shows up in results
    is_pending_ext_shared: bool,
    is_member: bool,
    is_private: bool,
    is_mpim: bool,
    last_read: Option<String>,
    is_open: Option<bool>,
    topic: Topic,
    purpose: Purpose,
    priority: u64,
    locale: Option<String>, // I'm not seeing this in the response, but it's in documentation, so I made it optional
}

#[derive(Deserialize, Debug)]
struct Im {
    id: String,
    created: u64,
    is_archived: bool,
    is_im: bool,
    is_org_shared: bool,
    user: String,
    is_user_deleted: bool,
    priority: u64,
}

#[derive(Deserialize, Debug)]
struct Topic {
    value: Option<String>,
    creator: Option<String>,
    last_set: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Purpose {
    value: Option<String>,
    creator: Option<String>,
    last_set: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    next_cursor: String,
}

fn get_token() -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string("TOKEN")?.parse::<String>()?.trim().to_string())
}

fn get_conversations(enabled_types: Vec<String>, exclude_archived: bool) -> Result<Vec<Conversation>, Box<dyn Error>> {
    let mut cursor = "".to_string();
    let mut conversations = vec![];
    let enabled_types = &enabled_types.join(",");
    loop {
        let mut result = get_conversations_page(enabled_types, exclude_archived, &cursor)?;
        cursor = result.response_metadata.next_cursor;
        conversations.append(&mut result.channels);
        if cursor == "" {
            break;
        }
    }

    return Ok(conversations);
}

fn get_conversations_page(enabled_types: &str, exclude_archived: bool, cursor: &str) -> Result<Conversations, Box<dyn Error>> {
    let mut response = Client::new()
        .get("https://slack.com/api/conversations.list")
        .query(&[
            ("cursor", cursor),
            ("exclude_archived", if exclude_archived { "true" } else { "false" }),
            ("limit", "1000"),
            // public_channel: #channel
            // private_channel: 🔒channel
            // mpim: 🧑🧑multi-person-direct-message
            // im: 🧑direct-message
            ("types", enabled_types)
        ])
        .header("Authorization", get_token()?)
        .send()?;

    let string = response.text()?;

    // println!("Text: {}", string);

    let result = serde_json::from_str::<ConversationsKind>(&string);

    match result? {
        ConversationsKind::Error(error) => Err(error)?,
        ConversationsKind::Conversations(conversations) => Ok(conversations),
    }
}

fn ls(types: [&str; 4], options: Option<&ArgMatches>) {
    println!("Retrieving conversations...");

    let enabled_types;
    let mut exclude_archived = false;
    if let Some(options) = options {
        enabled_types = if let Some(specified_types) = options.values_of_lossy("types") {
            specified_types
        } else {
            types.to_vec().iter().map(|s| s.to_string()).collect()
        };
        if options.is_present("exclude_archived") {
            exclude_archived = true;
        }
    } else {
        enabled_types = types.to_vec().iter().map(|s| s.to_string()).collect();
    };

    let conversations = get_conversations(enabled_types, exclude_archived).unwrap();

    println!("Available conversations:");
    for conversation in conversations {
        match conversation {
            Conversation::PublicChannel(convo) => {
                println!("- {} (#{})", convo.id, convo.name);
            },
            Conversation::PrivateChannel(convo) => {
                println!("- {} (🔒{})", convo.id, convo.name);
            },
            Conversation::Im(convo) => {
                println!("- {} (🧑{})", convo.id, convo.user);
            },
        }
    }
}
