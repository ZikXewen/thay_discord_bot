mod commands;
mod utils;

use std::ops::DerefMut;

use base64::Engine;

struct Data {
    http: crate::utils::http::Client,
    db_pool: deadpool_postgres::Pool,
    emojis: std::collections::HashMap<String, String>,
}

type Command = poise::Command<Data, anyhow::Error>;
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let discord_token =
        std::env::var("DISCORD_TOKEN").map_err(|_| anyhow::anyhow!("No DISCORD_TOKEN"))?;
    let framework = poise::Framework::builder()
        .setup(move |ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    http: crate::utils::http::Client::try_new()?,
                    db_pool: setup_db().await?,
                    emojis: setup_emojis(ctx).await?,
                })
            })
        })
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            ..Default::default()
        })
        .build();

    let intents = serenity::all::GatewayIntents::non_privileged(); // TODO: review intents
    let mut client = serenity::all::Client::builder(&discord_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}

mod embedded {
    refinery::embed_migrations!("migrations");
}

async fn setup_db() -> anyhow::Result<deadpool_postgres::Pool> {
    let user = std::env::var("DB_USER").map_err(|_| anyhow::anyhow!("No DB_USER"))?;
    let pass = std::env::var("DB_PASS").map_err(|_| anyhow::anyhow!("No DB_PASS"))?;
    let host = std::env::var("DB_HOST").map_err(|_| anyhow::anyhow!("No DB_HOST"))?;
    let name = std::env::var("DB_NAME").map_err(|_| anyhow::anyhow!("No DB_NAME"))?;

    let pg_config = tokio_postgres::Config::new()
        .host(host)
        .user(user)
        .password(pass)
        .dbname(name)
        .to_owned();
    let manager_config = deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast,
    };
    let manager =
        deadpool_postgres::Manager::from_config(pg_config, tokio_postgres::NoTls, manager_config);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(16)
        .build()
        .map_err(|_| anyhow::anyhow!("Could not create pool"))?;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| anyhow::anyhow!("Could not get a client"))?;
    let client = conn.deref_mut().deref_mut();
    embedded::migrations::runner()
        .run_async(client)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to migrate database"))?;

    Ok(pool)
}

async fn setup_emojis(
    ctx: &serenity::all::Context,
) -> anyhow::Result<std::collections::HashMap<String, String>> {
    let emojis = ctx
        .get_application_emojis()
        .await
        .map_err(|_| anyhow::anyhow!("Could not fetch emojis"))?;
    let iter = emojis.into_iter().map(|emoji| {
        let st = emoji.to_string();
        (emoji.name, st)
    });
    let mut map = std::collections::HashMap::from_iter(iter);

    let add_emojis = std::env::var("ADD_EMOJIS").is_ok();
    if add_emojis {
        println!("adding emojis");
        // NOTE: Fiddlesticks should be named FiddleSticks.png
        for img in std::fs::read_dir("./assets")? {
            let path = img?.path();
            if path.extension().is_none_or(|x| x != "png") {
                continue;
            }
            let name = path.file_stem().unwrap().to_string_lossy();
            if map.contains_key(name.as_ref()) {
                continue;
            }
            println!("adding {}", name.as_ref());
            let bytes = std::fs::read(path.to_str().unwrap())?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
            let b64 = format!("data:image/png;base64,{}", b64);
            let emoji = ctx.create_application_emoji(&name, &b64).await?;
            let st = emoji.to_string();
            println!("added {}", &emoji.name);
            map.insert(emoji.name, st);
        }
    }

    Ok(map)
}
