use crate::utils::{
    macros::{msg_err, reg_err},
    regions::Region,
    replies::say_text,
};

#[poise::command(slash_command, guild_only)]
pub async fn recent(ctx: crate::Context<'_>) -> anyhow::Result<()> {
    msg_err!(ctx.defer().await);
    let uid = ctx.author().id.get() as i64;
    let manager = reg_err!(
        ctx,
        ctx.data().db_pool.get().await,
        "Could not connect to database"
    );

    let row = reg_err!(
        ctx,
        manager
            .query_one(
                "SELECT region, riot_id, puuid FROM riot_auth WHERE uid = $1",
                &[&uid]
            )
            .await,
        "Could not find your account. **Please make sure you have bound an account with /bind first**"
    );
    let region: Region = row.get(0);
    let riot_id: String = row.get(1);
    let puuid: String = row.get(2);

    let http = ctx.data().http.clone();
    let res = reg_err!(
        ctx,
        http.get(format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?start=0&count=10",
            region.to_league_region(),
            puuid
        ))
        .send()
        .await,
        "Could not fetch matches"
    );
    let games: Vec<String> = reg_err!(ctx, res.json().await, "Failed to parse match ids");

    //TODO: maybe parallelize
    let mut wins = 0;
    for game in &games {
        let res = reg_err!(
            ctx,
            http.get(format!(
                "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
                region.to_league_region(),
                game
            ))
            .send()
            .await,
            "Could not fetch match info"
        );
        let mat: MatchDto = reg_err!(ctx, res.json().await, "Could not parse match info");
        if mat
            .info
            .participants
            .iter()
            .find(|par| par.puuid == puuid)
            .is_some_and(|par| par.win)
        {
            wins += 1;
        }
    }

    say_text(
        ctx,
        format!(
            "{} has won {} out of the last {} games",
            riot_id,
            wins,
            games.len()
        ),
    )
    .await;
    Ok(())
}

#[derive(serde::Deserialize)]
struct MatchDto {
    info: InfoDto,
}

#[derive(serde::Deserialize)]
struct InfoDto {
    participants: Vec<ParticipantDto>,
}

#[derive(serde::Deserialize)]
struct ParticipantDto {
    puuid: String,
    win: bool,
}
