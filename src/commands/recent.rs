use crate::utils::{
    macros::{msg_err, reg_err},
    replies::say_error,
};

#[poise::command(slash_command, guild_only)]
pub async fn recent(
    ctx: crate::Context<'_>,
    #[description = "Put 1 or leave empty for the most recent game."]
    #[min = 1]
    #[max = 20]
    game: Option<usize>,
    #[description = "The user whose game to retrieve."] user: Option<serenity::all::UserId>,
) -> anyhow::Result<()> {
    msg_err!(ctx.defer().await);
    let game = game.unwrap_or(1);
    if !(1..=20).contains(&game) {
        say_error(ctx, "`game` must be in range 1-20.").await;
        return Ok(());
    }

    let uid = user.unwrap_or(ctx.author().id).get() as i64;
    let manager = reg_err!(
        ctx,
        ctx.data().db_pool.get().await,
        "Could not connect to database"
    );

    let row = reg_err!(
        ctx,
        manager
            .query_one(
                "SELECT region, puuid FROM riot_auth WHERE uid = $1",
                &[&uid]
            )
            .await,
        "Could not find this user's account account. **Please make sure you have bound an account with /bind first**"
    );
    let region: crate::utils::regions::Region = row.get(0);
    let puuid: String = row.get(1);

    let http = ctx.data().http.clone();
    let res = reg_err!(
        ctx,
        http.get_with_riot_token(&format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?count=20",
            region.to_league_region(),
            puuid
        ))
        .send()
        .await,
        "Could not fetch matches"
    );
    let games: Vec<String> = reg_err!(ctx, res.json().await, "Failed to parse match ids");

    let res = reg_err!(
        ctx,
        http.get_with_riot_token(&format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
            region.to_league_region(),
            games[game - 1]
        ))
        .send()
        .await,
        "Could not fetch match info"
    );
    let MatchDto { info }: MatchDto = reg_err!(ctx, res.json().await, "Could not parse match info");

    let created = reg_err!(
        ctx,
        chrono::DateTime::from_timestamp_millis(info.game_creation)
            .ok_or(anyhow::anyhow!("Could not parse match info"))
    );
    let created = format!("{}", created.format("%d/%m/%Y %H:%M"));

    let queue = match info.queue_id {
        400 => "Draft Pick",
        420 => "Ranked Solo/Duo",
        440 => "Ranked Flex",
        450 => "ARAM",
        490 => "Quickplay",
        _ => "Custom/Special Mode",
    };

    let map = match info.map_id {
        11 => "Summoner's Rift",
        12 => "Howling Abyss",
        14 => "Butcher's Bridge",
        _ => "Unknown Map",
    };

    let mut blue = vec![];
    let mut red = vec![];
    let emojis = &ctx.data().emojis;
    for part in &info.participants {
        let partfmt = format!(
            "{}{} **{}#{}** - {}/{}/{}",
            emojis
                .get(&part.team_position)
                .unwrap_or(&part.team_position),
            emojis
                .get(&part.champion_name)
                .unwrap_or(&part.champion_name),
            part.riot_id_game_name,
            part.riot_id_tagline,
            part.kills,
            part.deaths,
            part.assists
        );
        if part.team_id == 100 {
            blue.push(partfmt);
        } else {
            red.push(partfmt);
        }
    }

    let blue_loss = (info.participants[0].team_id == 100) ^ info.participants[0].win;

    let embed = serenity::all::CreateEmbed::default()
        .title("Match Summary")
        .description(format!("{} - {}", queue, map))
        .color(serenity::all::colours::roles::BLUE)
        .field(
            "Match Info",
            format!(
                "**Game Start:** {}\n**Game Duration:** {}m {}s\n**Patch:** {}",
                created,
                info.game_duration / 60,
                info.game_duration % 60,
                info.game_version,
            ),
            false,
        )
        .field(
            if blue_loss {
                "Defeat (Blue Team)"
            } else {
                "Victory (Blue Team)"
            },
            blue.join("\n"),
            true,
        )
        .field(
            if blue_loss {
                "Victory (Red Team)"
            } else {
                "Defeat (Red Team)"
            },
            red.join("\n"),
            true,
        );

    let msg = poise::CreateReply::default().embed(embed);
    msg_err!(ctx.send(msg).await);
    Ok(())
}

#[derive(serde::Deserialize)]
struct MatchDto {
    info: InfoDto,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfoDto {
    game_creation: i64,
    game_duration: i64,
    game_version: String,
    map_id: u32,
    participants: Vec<ParticipantDto>,
    queue_id: u32,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParticipantDto {
    assists: u8,
    champion_name: String,
    deaths: u8,
    kills: u8,
    riot_id_game_name: String,
    riot_id_tagline: String,
    team_id: u32,
    team_position: String,
    win: bool,
}
