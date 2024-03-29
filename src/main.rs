extern crate reqwest;

use clap::{crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, LevelFilter};
use num_format::{Locale, ToFormattedString};
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::time::Instant;
use std::{fs, u64};

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
            .arg(
                Arg::with_name("SUBSTRING")
                    .help("Narrows results down to those that contain provided substring.")
                    .index(1)
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
    Error(ConversationsError),
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
impl Error for ConversationsError {}

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
    pending_shared: Vec<String>,  // I believe this should always be an empty array?
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
    pending_shared: Vec<String>,  // I believe this should always be an empty array?
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
    Ok(fs::read_to_string("TOKEN")?
        .parse::<String>()?
        .trim()
        .to_string())
}

fn get_conversations(
    enabled_types: Vec<String>,
    exclude_archived: bool,
) -> Result<Vec<Conversation>, Box<dyn Error>> {
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

fn get_conversations_page(
    enabled_types: &str,
    exclude_archived: bool,
    cursor: &str,
) -> Result<Conversations, Box<dyn Error>> {
    let mut response = Client::new()
        .get("https://slack.com/api/conversations.list")
        .query(&[
            ("cursor", cursor),
            (
                "exclude_archived",
                if exclude_archived { "true" } else { "false" },
            ),
            ("limit", "1000"),
            // public_channel: #channel
            // private_channel: 🔒channel
            // mpim: 🧑🧑multi-person-direct-message
            // im: 🧑direct-message
            ("types", enabled_types),
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum UserResult {
    Success(UserSuccess),
    Error(UserError),
}

#[derive(Deserialize, Debug)]
struct UserSuccess {
    ok: bool,
    user: User,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum User {
    Active(ActiveUser),
    Deleted(DeletedUser),
}

#[derive(Deserialize, Debug)]
struct ActiveUser {
    id: String,
    team_id: String,
    name: String,
    deleted: bool,
    color: String,
    real_name: String,
    tz: String,
    tz_label: String,
    tz_offset: i64,
    profile: Profile,
    is_admin: bool,
    is_owner: bool,
    is_primary_owner: bool,
    is_restricted: bool,
    is_ultra_restricted: bool,
    is_bot: bool,
    is_app_user: bool,
    updated: u64,
    has_2fa: bool,
}

#[derive(Deserialize, Debug)]
struct DeletedUser {
    id: String,
    team_id: String,
    name: String,
    deleted: bool,
    profile: Profile,
    is_bot: bool,
    is_app_user: bool,
    updated: u64,
}

#[derive(Deserialize, Debug)]
struct Profile {
    title: String, // Not in documentation
    phone: String, // Not in documentation
    skype: String, // Not in documentation
    real_name: String,
    real_name_normalized: String,
    display_name: String,
    display_name_normalized: String,
    status_text: String,
    status_emoji: String,
    status_expiration: u64, // Not in documentation
    avatar_hash: String,
    email: Option<String>,          // In documentation, but not response
    image_original: Option<String>, // In documentation, but not response
    image_24: String,
    image_32: String,
    image_48: String,
    image_72: String,
    image_192: String,
    image_512: String,
    status_text_canonical: String, // Not in documentation
    team: String,
}

#[derive(Deserialize, Debug)]
struct UserError {
    ok: bool,
    error: String,
}
impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
impl Error for UserError {}

fn get_user(user: String) -> Result<String, Box<dyn Error>> {
    let mut response = Client::new()
        .get("https://slack.com/api/users.info")
        .query(&[("user", user)])
        .header("Authorization", get_token()?)
        .send()?;

    let string = response.text()?;

    let result = serde_json::from_str::<UserResult>(&string);

    match result? {
        UserResult::Error(error) => Err(error)?,
        UserResult::Success(result) => Ok(match result.user {
            User::Active(user) => user.name,
            User::Deleted(user) => user.name,
        }),
    }
}

fn ls(types: [&str; 4], options: Option<&ArgMatches>) {
    let style = ProgressStyle::default_bar()
        .template(
            "{elapsed_precise} [{bar:40}] {pos:>7}/{len:7}\n           {prefix}\n           {msg}",
        )
        .progress_chars("=> ");

    let length = 4;
    let main_progress = ProgressBar::new(length);
    main_progress.set_style(style.clone());

    main_progress.set_prefix("Retrieving all conversations...");

    let enabled_types;
    let mut substring = "";
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
        if let Some(provided_substring) = options.value_of("SUBSTRING") {
            substring = provided_substring;
        }
    } else {
        enabled_types = types.to_vec().iter().map(|s| s.to_string()).collect();
    };

    let raw_conversations = get_conversations(enabled_types, exclude_archived).unwrap();

    std::thread::sleep(std::time::Duration::new(5, 0));

    main_progress.inc(1);
    main_progress.set_prefix("Retrieving metadata and normalizing conversations...");
    main_progress.set_length(raw_conversations.len() as u64 + length);

    let mut conversations = vec![];
    for conversation in raw_conversations {
        match conversation {
            Conversation::PublicChannel(convo) => {
                main_progress.set_message(&format!("Normalizing #{}", convo.name));
                conversations.push(NormalizedConversation {
                    id: convo.id,
                    type_identifier: "#".to_string(),
                    names: vec![convo.name],
                    is_archived: convo.is_archived,
                    is_deleted: false,
                });
            }
            Conversation::PrivateChannel(mut convo) => {
                if convo.name.starts_with("mpdm-") {
                    main_progress.set_message("Normalizing conversation with multiple members");
                    conversations.push(NormalizedConversation {
                        id: convo.id,
                        type_identifier: "&".to_string(),
                        names: convo
                            .name
                            .split_off(5)
                            .rsplitn(2, "-")
                            .last()
                            .unwrap()
                            .split("--")
                            .map(|s| s.to_string())
                            .collect(),
                        is_archived: convo.is_archived,
                        is_deleted: false,
                    });
                } else {
                    main_progress
                        .set_message(&format!("Normalizing private channel #{}", convo.name));
                    conversations.push(NormalizedConversation {
                        id: convo.id,
                        type_identifier: "!".to_string(),
                        names: vec![convo.name],
                        is_archived: convo.is_archived,
                        is_deleted: false,
                    });
                }
            }
            Conversation::Im(convo) => {
                main_progress.set_message(&format!("Retrieving metadata for user {}", convo.user));
                let name = get_user(convo.user).unwrap();
                main_progress.tick();
                main_progress.set_message(&format!("Normalizing conversation with @{}", name));
                conversations.push(NormalizedConversation {
                    id: convo.id,
                    type_identifier: "@".to_string(),
                    names: vec![name],
                    is_archived: convo.is_archived,
                    is_deleted: convo.is_user_deleted,
                });
            }
        }
        main_progress.inc(1);
    }

    if substring != "" {
        main_progress.set_prefix(&format!(
            "Filtering conversations down to those that contain `{}`...",
            substring
        ));

        conversations = conversations
            .into_iter()
            .filter(|convo| {
                for name in &convo.names {
                    if name.contains(substring) {
                        return true;
                    }
                }
                false
            })
            .collect::<Vec<NormalizedConversation>>();
    }

    main_progress.inc(1);
    main_progress.set_prefix("Sorting names in multi-person DMs...");

    for conversation in &mut conversations {
        conversation.names.sort_unstable();
    }

    main_progress.inc(1);
    main_progress.set_prefix("Sorting conversations by type and name...");

    conversations.sort_unstable_by(|a, b| a.names.partial_cmp(&b.names).unwrap());
    conversations.sort_by(|a, b| a.type_identifier.partial_cmp(&b.type_identifier).unwrap());

    main_progress.inc(1);
    main_progress.finish_and_clear();

    if substring != "" {
        println!("All conversations you have access to:");
    } else {
        println!(
            "All conversations with names that contain `{}` that you have access to:",
            substring
        );
    }

    for conversation in conversations {
        let (icon, color) = if conversation.is_deleted {
            ("🗑", Color::Red)
        } else if conversation.is_archived {
            ("🗄", Color::Yellow)
        } else {
            ("🗒", Color::White)
        };
        println!(
            "{}",
            format!(
                "{} {}: {}{}",
                icon,
                conversation.id.bold(),
                &conversation.type_identifier,
                conversation
                    .names
                    .join(&format!(", {}", &conversation.type_identifier))
            )
            .color(color)
        );
    }

    #[derive(Debug)]
    struct NormalizedConversation {
        id: String,
        type_identifier: String,
        names: Vec<String>,
        is_archived: bool,
        is_deleted: bool,
    }
}
