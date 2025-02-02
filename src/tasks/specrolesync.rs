use std::num::NonZeroU64;

use crate::config;

pub enum SpecialRole {
    BugHunter,
}

struct SpecRoleSync {
    user_id: NonZeroU64,
    col: SpecialRole,
}

pub async fn spec_role_sync(
    pool: &sqlx::PgPool,
    cache_http: &crate::impls::cache::CacheHttpImpl,
) -> Result<(), crate::Error> {
    // Now actually resync
    let mut resync = Vec::new();

    let bug_hunter_role = poise::serenity_prelude::RoleId(config::CONFIG.roles.bug_hunters);

    {
        if let Some(guild) = cache_http.cache.guild(config::CONFIG.servers.main) {
            // Get all members with the bug hunter role
            for (_, member) in guild.members.iter() {
                if member.roles.contains(&bug_hunter_role) {
                    resync.push(SpecRoleSync {
                        user_id: member.user.id.0,
                        col: SpecialRole::BugHunter,
                    });
                }
            }
        } else {
            log::warn!("Failed to get guild");
        }
    }

    // Create a transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("Error creating transaction: {:?}", e))?;

    sqlx::query!("UPDATE users SET bug_hunters = false")
        .execute(&mut tx)
        .await
        .map_err(|e| format!("Error updating users: {:?}", e))?;

    for user in resync {
        match user.col {
            SpecialRole::BugHunter => {
                sqlx::query!(
                    "
                    UPDATE users SET bug_hunters = true WHERE user_id = $1",
                    user.user_id.to_string()
                )
                .execute(&mut tx)
                .await
                .map_err(|e| format!("Error updating users: {:?}", e))?;
            }
        }
    }

    tx.commit().await?;

    Ok(())
}
