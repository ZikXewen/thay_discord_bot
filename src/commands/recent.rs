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

    let created = reg_err!(
        ctx,
        chrono::DateTime::from_timestamp_millis(info.game_creation)
            .ok_or(anyhow::anyhow!("Could not parse match info"))
    );
    let created = format!("{}", created.format("%d/%m/%Y %H:%M"));
    let match_info = format!(
        "**Game Start:** {}\n**Game Duration:** {}m {}s\n**Patch:** {}",
        created,
        info.game_duration / 60,
        info.game_duration % 60,
        info.game_version,
    );

    let blue_loss = (info.participants[0].team_id == 100) ^ info.participants[0].win;
    let blue_header = if blue_loss {
        "Defeat (Blue Team)"
    } else {
        "Victory (Blue Team)"
    };
    let red_header = if blue_loss {
        "Victory (Red Team)"
    } else {
        "Defeat (Red Team)"
    };
    let mut blue = vec![];
    let mut red = vec![];
    let emojis = &ctx.data().emojis;
    for part in &info.participants {
        let partfmt = format!(
            "{}{} **{}#{}** - {}/{}/{}",
            get_emoji(emojis, &part.team_position),
            get_emoji(emojis, &part.champion_name),
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

    let part = reg_err!(
        ctx,
        info.participants
            .iter()
            .find(|p| p.puuid == puuid)
            .ok_or(anyhow::anyhow!("Could not parse match info"))
    );
    let part_info = format!(
        "{}{} **{}#{}**",
        get_emoji(emojis, &part.team_position),
        get_emoji(emojis, &part.champion_name),
        part.riot_id_game_name,
        part.riot_id_tagline,
    );

    let kda = if part.deaths == 0 {
        "Perfect".to_string()
    } else {
        ((part.kills + part.assists) as f32 / part.deaths as f32).to_string()
    };
    let kda = format!(
        "{}/{}/{} ({} KDA)",
        part.kills, part.deaths, part.assists, kda
    );

    let cs = part.neutral_minions_killed + part.total_minions_killed;
    let cspm = (cs * 60) as f32 / info.game_duration as f32;

    #[rustfmt::skip]
    let runes = format!(
        "{}{}{}{} {}{}{} {}{}{}",
        get_emoji(emojis, &format!("r{}", part.perks.styles[0].style)),
        get_emoji(emojis, &format!("r{}", part.perks.styles[0].selections[0].perk)),
        get_emoji(emojis, &format!("r{}", part.perks.styles[0].selections[1].perk)),
        get_emoji(emojis, &format!("r{}", part.perks.styles[0].selections[2].perk)),
        get_emoji(emojis, &format!("r{}", part.perks.styles[1].style)),
        get_emoji(emojis, &format!("r{}", part.perks.styles[1].selections[0].perk)),
        get_emoji(emojis, &format!("r{}", part.perks.styles[1].selections[1].perk)),
        get_emoji(emojis, &format!("r{}", part.perks.stat_perks.offense)),
        get_emoji(emojis, &format!("r{}", part.perks.stat_perks.defense)),
        get_emoji(emojis, &format!("r{}", part.perks.stat_perks.flex)),
    );

    let items = [
        part.item0, part.item1, part.item2, part.item3, part.item4, part.item5, part.item6,
    ]
    .into_iter()
    .flat_map(|item| {
        if item == 0 {
            None
        } else {
            Some(get_emoji(emojis, &item.to_string()).to_string())
        }
    })
    .collect::<Vec<_>>()
    .join("");

    let embed = serenity::all::CreateEmbed::default()
        .title("Match Summary")
        .description(format!("{} - {}", queue, map))
        .color(serenity::all::colours::roles::BLUE)
        .field("Match Info", match_info, false)
        .field(blue_header, blue.join("\n"), true)
        .field(red_header, red.join("\n"), true)
        .field("Personal Performance", part_info, false)
        .field("KDA", kda, true)
        .field("CS", format!("{} ({:.1}/min)", cs, cspm), true)
        .field("Gold Earned", part.gold_earned.to_string(), true)
        .field(
            "Total Damage Dealt",
            part.total_damage_dealt.to_string(),
            true,
        )
        .field(
            "Damage Dealt to Champions",
            part.total_damage_dealt_to_champions.to_string(),
            true,
        )
        .field(
            "Total Damage Taken",
            part.total_damage_taken.to_string(),
            true,
        )
        .field("Runes", runes, false)
        .field("Items", items, false);

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
    assists: u32,
    champion_name: String,
    deaths: u32,
    gold_earned: u32,
    item0: u32,
    item1: u32,
    item2: u32,
    item3: u32,
    item4: u32,
    item5: u32,
    item6: u32,
    kills: u32,
    neutral_minions_killed: u32,
    perks: PerksDto,
    puuid: String,
    riot_id_game_name: String,
    riot_id_tagline: String,
    team_id: u32,
    team_position: String,
    total_damage_dealt: u32,
    total_damage_dealt_to_champions: u32,
    total_damage_taken: u32,
    total_minions_killed: u32,
    win: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PerksDto {
    stat_perks: PerkStatsDto,
    styles: Vec<PerkStyleDto>,
}

#[derive(serde::Deserialize)]
struct PerkStatsDto {
    defense: u32,
    flex: u32,
    offense: u32,
}

#[derive(serde::Deserialize)]
struct PerkStyleDto {
    selections: Vec<PerkStyleSelectionDto>,
    style: u32,
}

#[derive(serde::Deserialize)]
struct PerkStyleSelectionDto {
    perk: u32,
}

fn get_emoji<'a>(
    emojis: &'a std::collections::HashMap<String, String>,
    name: &'a String,
) -> &'a String {
    emojis.get(name).unwrap_or(name)
}
