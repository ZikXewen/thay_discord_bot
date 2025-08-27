import base64
import os
import requests

cdragon_url = "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/"
discord_url = "https://discord.com/api/applications/"
discord_token = os.environ.get("DISCORD_TOKEN")
discord_headers = {'Authorization': 'Bot ' + discord_token}

with requests.get(discord_url + "@me", headers=discord_headers) as res:
    res.raise_for_status()
    app_id = res.json()["id"]

with requests.get(f"{discord_url}{app_id}/emojis", headers=discord_headers) as res:
    res.raise_for_status()
    old_emojis = {emoji["name"]: emoji["id"] for emoji in res.json()["items"]}

new_emojis = set()

def add_emoji(name, path):
    new_emojis.add(name)
    if name in old_emojis:
        return
    print("adding " + name)
    with requests.get(path.replace("/lol-game-data/assets/", cdragon_url).lower()) as res:
        res.raise_for_status()
        image = "data:image/png;base64," + base64.b64encode(res.content).decode('utf-8')
    with requests.post(f"{discord_url}{app_id}/emojis", json={'name': name, 'image': image}, headers=discord_headers) as res:
        res.raise_for_status()

# Champions
with requests.get(cdragon_url + "v1/champion-summary.json") as res:
    res.raise_for_status()
    for champ in res.json():
        add_emoji(champ["alias"], champ["squarePortraitPath"])

# Items
with requests.get(cdragon_url + "v1/items.json") as res:
    res.raise_for_status()
    for item in res.json():
        add_emoji(str(item["id"]), item["iconPath"])

# Rune Trees
with requests.get(cdragon_url + "v1/perkstyles.json") as res:
    res.raise_for_status()
    for tree in res.json()["styles"]:
        add_emoji(f'r{tree["id"]}', tree["iconPath"])

# Runes
with requests.get(cdragon_url + "v1/perks.json") as res:
    res.raise_for_status()
    for rune in res.json():
        add_emoji(f'r{rune["id"]}', rune["iconPath"])

# Roles
for role in ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"]:
    add_emoji(role, f"/lol-game-data/assets/../../../rcp-fe-lol-clash/global/default/assets/images/position-selector/positions/icon-position-{role}.png")

for name, id in old_emojis.items():
    if name not in new_emojis:
        print("removing " + name)
        with requests.delete(f"{discord_url}{app_id}/emojis/{id}", headers=discord_headers) as res:
            res.raise_for_status()
