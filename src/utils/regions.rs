#[derive(Debug, poise::ChoiceParameter, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "riot_region")]
#[allow(clippy::upper_case_acronyms)]
pub enum Region {
    #[name = "North America"]
    NA,
    #[name = "Brazil"]
    BR,
    #[name = "Latin America North"]
    LAN,
    #[name = "Latin America South"]
    LAS,
    #[name = "Korea"]
    KR,
    #[name = "Japan"]
    JP,
    #[name = "Europe Northeast"]
    EUNE,
    #[name = "Europe West"]
    EUW,
    #[name = "Middle East"]
    ME1,
    #[name = "Turkiye"]
    TR,
    #[name = "Russia"]
    RU,
    #[name = "Oceania"]
    OCE,
    #[name = "South East Asia"]
    SG2,
    #[name = "Taiwan"]
    TW2,
    #[name = "Vietnam"]
    VN2,
}

impl Region {
    pub fn to_riot_region(&self) -> &'static str {
        use Region::*;
        match self {
            NA | BR | LAN | LAS => "americas",
            KR | JP | OCE | SG2 | TW2 | VN2 => "asia",
            EUNE | EUW | ME1 | TR | RU => "europe",
        }
    }
    pub fn to_league_region(&self) -> &'static str {
        use Region::*;
        match self {
            NA | BR | LAN | LAS => "americas",
            KR | JP => "asia",
            EUNE | EUW | ME1 | TR | RU => "europe",
            OCE | SG2 | TW2 | VN2 => "sea",
        }
    }
}
