use crate::utils::{
    macros::{msg_err, reg_err},
    regions::Region,
    replies::say_text,
};

#[poise::command(slash_command, guild_only)]
pub async fn bind(
    ctx: crate::Context<'_>,
    #[description = "region of your riot account"] region: Region,
    #[description = "username#tag"] riot_id: String,
) -> anyhow::Result<()> {
    msg_err!(ctx.defer().await);
    let uid = ctx.author().id.get() as i64;
    let manager = reg_err!(
        ctx,
        ctx.data().db_pool.get().await,
        "Could not connect to database"
    );

    let http = ctx.data().http.clone();
    let res = reg_err!(
        ctx,
        http.get(format!(
            "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}",
            region.to_riot_region(),
            riot_id.replace("#", "/")
        ))
        .send()
        .await,
        "Could not fetch account info"
    );
    let account: AccountDto = reg_err!(ctx, res.json().await, "Could not parse account info");

    reg_err!(
        ctx,
        manager
            .query(
                "
            INSERT INTO riot_auth (uid, region, riot_id, puuid)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (uid) DO UPDATE
            SET region = EXCLUDED.region, riot_id = EXCLUDED.riot_id, puuid = EXCLUDED.puuid",
                &[&uid, &region, &riot_id, &account.puuid]
            )
            .await,
        "Could not update database"
    );
    say_text(ctx, "Riot account linked successfully").await;
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct AccountDto {
    puuid: String,
    game_name: String,
    tag_line: String,
}
