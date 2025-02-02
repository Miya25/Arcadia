use std::str::FromStr;
use std::time::Duration;

use poise::serenity_prelude::{
    ButtonStyle, CreateActionRow, CreateButton, CreateInputText, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateQuickModal, InputTextStyle, ModalInteraction,
};
use poise::CreateReply;
use strum::VariantNames;

use crate::{Context, Error};

async fn autocomplete(_ctx: Context<'_>, partial: &str) -> Vec<poise::AutocompleteChoice<String>> {
    let mut choices = Vec::new();

    for m in super::core::RPCMethod::VARIANTS {
        if partial.is_empty() || m.contains(partial) {
            choices.push(poise::AutocompleteChoice {
                name: m.to_string(),
                value: m.to_string(),
            });
        }
    }

    choices
}

fn parse_bool(v: &str) -> Result<bool, Error> {
    match v.to_lowercase().as_str() {
        "true" | "t" | "y" => Ok(true),
        "false" | "f" | "n" => Ok(false),
        _ => Err("Invalid boolean".into()),
    }
}

fn parse_hrs(v: &str) -> Result<i32, Error> {
    // Split v into time and unit
    let data = v.split(' ').collect::<Vec<&str>>();

    if data.len() != 2 {
        return Err(
            "Invalid time format. Format must be WITH A SPACE BETWEEN THE NUMBER AND THE UNIT"
                .into(),
        );
    }

    let (time, unit) = (data[0], data[1]);

    let time = time.parse::<i32>()?;

    match unit {
        "years" | "year" | "y" => Ok(time * 365 * 24),
        "months" | "month" | "mo" | "m" => Ok(time * 30 * 24),
        "weeks" | "week" | "w" => Ok(time * 7 * 24),
        "days" | "day" | "d" => Ok(time * 24),
        "hours" | "hour" | "hrs" | "hr" | "h" => Ok(time),
        _ => Err("Invalid time format. Unit must be years, months, weeks, days or hours".into()),
    }
}

struct GetResp {
    method: super::core::RPCMethod,
    interaction: ModalInteraction,
}

#[poise::command(
    prefix_command,
    slash_command,
    check = "crate::checks::is_staff"
)]
pub async fn rpc(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete"] method: String,
) -> Result<(), Error> {
    // Creates a "blank" RPCMethod
    let variant = super::core::RPCMethod::from_str(&method)?;
    let discord = ctx.discord();

    let rpc_method = {
        // Send modal button
        let builder = CreateReply::default()
        .content("OK, we just need some extra information first, please click the below button to launch a modal asking for more information")
        .components(
            vec![
                CreateActionRow::Buttons(
                    vec![
                        CreateButton::new("next")
                        .label("Next")
                        .style(ButtonStyle::Primary),
                        CreateButton::new("cancel")
                        .label("Cancel")
                        .style(ButtonStyle::Danger)
                    ]
                )
            ]
        );

        let mut msg = ctx.send(builder.clone()).await?.into_message().await?;

        let interaction = msg
            .await_component_interaction(ctx.discord())
            .author_id(ctx.author().id)
            .timeout(Duration::from_secs(120))
            .await;

        if let Some(m) = &interaction {
            let id = &m.data.custom_id;

            msg.edit(ctx.discord(), builder.to_prefix_edit().components(vec![]))
                .await?; // remove buttons after button press

            if id == "cancel" {
                return Ok(());
            }

            match variant {
                super::core::RPCMethod::BotClaim { .. } => {
                    let qm = CreateQuickModal::new("Claim Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Force Claim [Y/N]",
                            "fclaim",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, force) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotClaim {
                                bot_id: bot_id.to_string(),
                                force: parse_bool(force)?,
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotUnclaim { .. } => {
                    let qm = CreateQuickModal::new("Unclaim Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotUnclaim {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotApprove { .. } => {
                    let qm = CreateQuickModal::new("Approve Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotApprove {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotDeny { .. } => {
                    let qm = CreateQuickModal::new("Deny Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotDeny {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotVoteReset { .. } => {
                    let qm = CreateQuickModal::new("Vote Reset Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotVoteReset {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotVoteResetAll { .. } => {
                    let qm = CreateQuickModal::new("Vote Reset All Bots").field(
                        CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason"),
                    );

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let reason = &inputs[0];

                        GetResp {
                            method: super::core::RPCMethod::BotVoteResetAll {
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotUnverify { .. } => {
                    let qm = CreateQuickModal::new("Unverify Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(
                            CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                                .placeholder("You must give proof"),
                        );

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotUnverify {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotPremiumAdd { .. } => {
                    let qm = CreateQuickModal::new("Add Bot To Premium")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(
                            CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                                .placeholder("You must give proof"),
                        )
                        .field(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                "Time Period",
                                "time_period",
                            )
                            .placeholder(
                                "Format: INTEGER UNIT, e.g. 1 day, 2 weeks, 3 months, 4 years",
                            ),
                        );

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason, time_period_str) =
                            (&inputs[0], &inputs[1], &inputs[2]);

                        GetResp {
                            method: super::core::RPCMethod::BotPremiumAdd {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                                time_period_hours: match parse_hrs(time_period_str) {
                                    Ok(hrs) => hrs,
                                    Err(e) => {
                                        resp.interaction
                                            .create_response(
                                                ctx,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::default()
                                                        .content(format!("**{}**", e)),
                                                ),
                                            )
                                            .await?;

                                        return Ok(());
                                    }
                                },
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotPremiumRemove { .. } => {
                    let qm = CreateQuickModal::new("Remove Bot From Premium")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(
                            CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                                .placeholder("You must give proof"),
                        );

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotPremiumRemove {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotVoteBanAdd { .. } => {
                    let qm = CreateQuickModal::new("Vote Ban Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotVoteBanAdd {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotVoteBanRemove { .. } => {
                    let qm = CreateQuickModal::new("Vote Ban Remove Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotVoteBanRemove {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotForceRemove { .. } => {
                    let qm = CreateQuickModal::new("Force Remove Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ))
                        .field(
                            CreateInputText::new(InputTextStyle::Short, "Kick?", "kick")
                                .placeholder("T/F"),
                        );

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason, kick) = (&inputs[0], &inputs[1], &inputs[2]);

                        GetResp {
                            method: super::core::RPCMethod::BotForceRemove {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                                kick: match parse_bool(kick) {
                                    Ok(b) => b,
                                    Err(e) => {
                                        resp.interaction
                                            .create_response(
                                                ctx,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::default()
                                                        .content(format!(
                                                            "**Error parsing `kick`: {}**",
                                                            e
                                                        )),
                                                ),
                                            )
                                            .await?;

                                        return Ok(());
                                    }
                                },
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotCertifyAdd { .. } => {
                    let qm = CreateQuickModal::new("Certify Bot (not recommended)")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotCertifyAdd {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotCertifyRemove { .. } => {
                    let qm = CreateQuickModal::new("Uncertify Bot")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason) = (&inputs[0], &inputs[1]);

                        GetResp {
                            method: super::core::RPCMethod::BotCertifyRemove {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotVoteCountSet { .. } => {
                    let qm = CreateQuickModal::new("Set Bot Vote Count")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Count",
                            "count",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason, count_str) = (&inputs[0], &inputs[1], &inputs[2]);

                        GetResp {
                            method: super::core::RPCMethod::BotVoteCountSet {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                                count: match count_str.parse::<i32>() {
                                    Ok(c) => c,
                                    Err(e) => {
                                        resp.interaction
                                            .create_response(
                                                ctx,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::default()
                                                        .content(format!(
                                                            "**Error parsing `count`: {}**",
                                                            e
                                                        )),
                                                ),
                                            )
                                            .await?;

                                        return Ok(());
                                    }
                                },
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotTransferOwnershipUser { .. } => {
                    let qm = CreateQuickModal::new("Transfer Bot Ownership")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "New Owner ID",
                            "new_owner",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason, new_owner) = (&inputs[0], &inputs[1], &inputs[2]);

                        GetResp {
                            method: super::core::RPCMethod::BotTransferOwnershipUser {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                                new_owner: new_owner.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::BotTransferOwnershipTeam { .. } => {
                    let qm = CreateQuickModal::new("Transfer Bot Ownership to Team")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Bot ID",
                            "bot_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "New Team ID",
                            "team_id",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (bot_id, reason, team_id) = (&inputs[0], &inputs[1], &inputs[2]);

                        GetResp {
                            method: super::core::RPCMethod::BotTransferOwnershipTeam {
                                bot_id: bot_id.to_string(),
                                reason: reason.to_string(),
                                new_team: team_id.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
                super::core::RPCMethod::TeamNameEdit { .. } => {
                    let qm = CreateQuickModal::new("Team Name Edit")
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "Team ID",
                            "team_id",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Short,
                            "New Team Name",
                            "new_name",
                        ))
                        .field(CreateInputText::new(
                            InputTextStyle::Paragraph,
                            "Reason",
                            "reason",
                        ));

                    if let Some(resp) = m.quick_modal(discord, qm).await? {
                        let inputs = resp.inputs;
                        let (team_id, new_name, reason) = (&inputs[0], &inputs[1], &inputs[2]);

                        GetResp {
                            method: super::core::RPCMethod::TeamNameEdit {
                                team_id: team_id.to_string(),
                                new_name: new_name.to_string(),
                                reason: reason.to_string(),
                            },
                            interaction: resp.interaction,
                        }
                    } else {
                        return Err("No response".into());
                    }
                }
            }
        } else {
            msg.edit(ctx.discord(), builder.to_prefix_edit().components(vec![]))
                .await?; // remove buttons after timeout
            return Ok(());
        }
    };

    if rpc_method.method.to_string() != method {
        return Err(format!(
            "Internal error: method ({}) != variant ({})",
            rpc_method.method, method
        )
        .into());
    }

    let data = ctx.data();

    match rpc_method
        .method
        .handle(crate::rpc::core::RPCHandle {
            cache_http: data.cache_http.clone(),
            pool: data.pool.clone(),
            user_id: ctx.author().id.to_string(),
        })
        .await
    {
        Ok(resp) => match resp {
            super::core::RPCSuccess::NoContent => {
                rpc_method
                    .interaction
                    .create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default().content(format!(
                                "Successfully performed the operation required: `{}`",
                                rpc_method.method
                            )),
                        ),
                    )
                    .await?;

                Ok(())
            }
            super::core::RPCSuccess::Content(msg) => {
                rpc_method
                    .interaction
                    .create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default().content(format!(
                                "Successfully performed the operation required: `{}`\n**{}**",
                                rpc_method.method, msg
                            )),
                        ),
                    )
                    .await?;
                Ok(())
            }
        },
        Err(e) => {
            rpc_method
                .interaction
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::default().content(format!(
                            "Error performing `{}`: **{}**",
                            rpc_method.method, e
                        )),
                    ),
                )
                .await?;
            Ok(())
        }
    }
}
