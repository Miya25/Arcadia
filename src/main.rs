use log::{error, info};
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateEmbed, CreateMessage, FullEvent, GuildId, RoleId, Timestamp,
};
use sqlx::postgres::PgPoolOptions;

use crate::impls::cache::CacheHttpImpl;

mod admin;
mod botowners;
mod checks;
mod config;
mod explain;
mod help;
mod impls;
mod rpc;
mod staff;
mod stats;
mod tasks;
mod testing;
mod test;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {
    pool: sqlx::PgPool,
    cache_http: CacheHttpImpl,
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
            let err = ctx
                .say(format!(
                    "There was an error running this command: {}",
                    error
                ))
                .await;

            if let Err(e) = err {
                error!("SQLX Error: {}", e);
            }
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx } => {
            error!(
                "[Possible] error in command `{}`: {:?}",
                ctx.command().name,
                error,
            );
            if let Some(error) = error {
                error!("Error in command `{}`: {:?}", ctx.command().name, error,);
                let err = ctx
                    .say(format!(
                        "Whoa there, do you have permission to do this?: {}",
                        error
                    ))
                    .await;

                if let Err(e) = err {
                    error!("Error while sending error message: {}", e);
                }
            } else {
                let err = ctx
                    .say("You don't have permission to do this but we couldn't figure out why...")
                    .await;

                if let Err(e) = err {
                    error!("Error while sending error message: {}", e);
                }
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

async fn event_listener(event: &FullEvent, user_data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate {
            interaction,
            ctx: _,
        } => {
            info!("Interaction received: {:?}", interaction.id());
        }
        FullEvent::CacheReady { ctx: _, guilds } => {
            info!("Cache ready with {} guilds", guilds.len());
        }
        FullEvent::Ready {
            data_about_bot,
            ctx: _,
        } => {
            info!(
                "{} is ready! Doing some minor DB fixes",
                data_about_bot.user.name
            );

            sqlx::query!(
                "UPDATE bots SET claimed_by = NULL, type = 'pending' WHERE LOWER(claimed_by) = 'none'",
            )
            .execute(&user_data.pool)
            .await?;

            // Start RPC
            tokio::task::spawn(rpc::server::rpc_init(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
            ));

            tokio::task::spawn(crate::tasks::taskcat::start_all_tasks(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
            ));
        }
        FullEvent::GuildMemberAddition { new_member, ctx } => {
            if new_member.guild_id.0 == config::CONFIG.servers.main && new_member.user.bot {
                // Check if new member is in testing server
                let member = ctx.cache.member_field(
                    GuildId(config::CONFIG.servers.testing),
                    new_member.user.id,
                    |m| m.user.id,
                );

                if member.is_some() {
                    // If so, move them to main server
                    GuildId(config::CONFIG.servers.testing)
                        .kick_with_reason(&ctx, new_member.user.id, "Added to main server")
                        .await?;
                }

                // Send member join message
                ChannelId(config::CONFIG.channels.system)
                .send_message(
                    &ctx,
                    CreateMessage::new()
                    .embed(
                        CreateEmbed::default()
                        .title("__**New Bot Added**__")
                        .description(
                            format!(
                                "Bot <@{}> ({}) has joined the server and has been given the `Bots` role.",
                                new_member.user.id,
                                new_member.user.name
                            )
                        )
                        .color(0x00ff00)
                        .thumbnail(new_member.user.face())
                        .timestamp(Timestamp::now())
                    )
                )
                .await?;

                // Give bot role
                ctx.http
                    .add_member_role(
                        GuildId(config::CONFIG.servers.main),
                        new_member.user.id,
                        RoleId(config::CONFIG.roles.bot_role),
                        Some("Bot added to server"),
                    )
                    .await?;
            }

            if new_member.guild_id.0 == config::CONFIG.servers.main && !new_member.user.bot {
                // Send member join message
                ChannelId(config::CONFIG.channels.system)
                .send_message(
                    &ctx,
                    CreateMessage::new()
                    .embed(
                        CreateEmbed::default()
                        .title("__**New User**__")
                        .description(
                            format!(
                                "Hmmmm... looks like <@{}> ({}) has strolled in. Can we trust them?",
                                new_member.user.id,
                                new_member.user.name
                            )
                        )
                        .color(0x00ff00)
                        .thumbnail(new_member.user.face())
                        .timestamp(Timestamp::now())
                    )
                )
                .await?;
            }
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    const MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here

    std::env::set_var("RUST_LOG", "bot=info, moka=error");

    env_logger::init();

    info!("Proxy URL: {}", config::CONFIG.proxy_url);

    let http = serenity::HttpBuilder::new(&config::CONFIG.token)
        .proxy(config::CONFIG.proxy_url.clone())
        .ratelimiter_disabled(true)
        .build();

    let client_builder =
        serenity::ClientBuilder::new_with_http(http, serenity::GatewayIntents::all());

    let framework = poise::Framework::new(
        poise::FrameworkOptions {
            initialize_owners: true,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("ibb!".into()),
                ..poise::PrefixFrameworkOptions::default()
            },
            listener: |event, _ctx, user_data| Box::pin(event_listener(event, user_data)),
            commands: vec![
                age(),
                register(),
                help::simplehelp(),
                help::help(),
                explain::explainme(),
                staff::staff(),
                testing::invite(),
                testing::claim(),
                testing::unclaim(),
                testing::queue(),
                testing::approve(),
                testing::deny(),
                testing::staffguide(),
                admin::rpcidentify(),
                admin::rpclock(),
                admin::protectdeploy(),
                admin::unprotectdeploy(),
                admin::updprod(),
                admin::uninvitedbots(),
                stats::stats(),
                botowners::getbotroles(),
                rpc::command::rpc(),
                test::modaltest(),
            ],
            /// This code is run before every command
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Executing command {} for user {} ({})...",
                        ctx.command().qualified_name,
                        ctx.author().name,
                        ctx.author().id
                    );
                })
            },
            /// This code is run after every command returns Ok
            post_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Done executing command {} for user {} ({})...",
                        ctx.command().qualified_name,
                        ctx.author().name,
                        ctx.author().id
                    );
                })
            },
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        },
        move |ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    cache_http: CacheHttpImpl {
                        cache: ctx.cache.clone(),
                        http: ctx.http.clone(),
                    },
                    pool: PgPoolOptions::new()
                        .max_connections(MAX_CONNECTIONS)
                        .connect(&config::CONFIG.database_url)
                        .await
                        .expect("Could not initialize connection"),
                })
            })
        },
    );

    let mut client = client_builder
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
