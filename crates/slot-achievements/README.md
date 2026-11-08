# RetroAchievements

slotPlus earns **softcore RetroAchievements for Game Boy, Game Boy Color, Game Boy
Advance, WonderSwan, WonderSwan Color and Atari Lynx games**. Earning one shows a short banner in Slot's style, the achievement's title and
points on one line and its description below, with its own badge, and plays a quiet chime that
follows the game's volume. There is no achievement browser, leaderboards or hardcore mode.
Save states, resume, rewind and fast-forward all still work.

Ported from Prashant Vaibhav's fork of Slot (github.com/pvaibhav/slot), GPL-3.0, with the
official rcheevos evaluator (MIT, `vendor/rcheevos`).

## Setting it up

The easy way: Settings → RetroAchievements. Enter your **Username** and **Password** on the
keyboard, join Wi-Fi from Settings → General → Wi-Fi, and you're set; the service starts again with
the new account straight away. The page also has **Encore** and shows **Cheevo Mode**, which
is always Casual (softcore): hardcore needs a live connection the whole time and no save
states, rewind or fast-forward, which slotPlus doesn't do.

The page reads and writes `Config/retroachievements.toml`, keeping anything else in it, so
the file still works by hand. Copy `Config/retroachievements.toml.example` to it and fill in
your account:

```toml
enabled = true
username = "your-ra-username"
password = "your-ra-password"
```

Join a Wi-Fi network from Settings → General → Wi-Fi, then restart slotPlus. After the first
login the account's token is kept under `Saves/RetroAchievements/` and the `password` line
can be removed. `token = "..."` works instead of a password; it is an emulator login token,
not a Web API key. Omit the file or set `enabled = false` to turn the feature off.

`encore = true` lets achievements you already have go off again (banner and chime). Nothing
is sent to RetroAchievements for those.

Typing a username or password in Settings turns achievements on and drops a `token` line
from the file, since a token would otherwise win over the password just typed.

Treat the file and the token as credentials. Neither is logged. Like Slot's own Wi-Fi
password, they are on the card, so anyone you let into Gamebit can reach them.

## WonderSwan and Lynx

Neither core publishes a memory map, so slotPlus lays their memory out the way RetroAchievements
expects: the 64 KB of system RAM at 0x0000, and for the WonderSwan the cart's save RAM or
EEPROM from 0x10000. A `.lnx` game is hashed without its 64-byte header, as RetroAchievements
does; `.lyx`, `.ws` and `.wsc` games are hashed whole.

## Downloading sets

A game earns from its achievement set, which has to be on the card first. In Settings →
RetroAchievements, **Download Achievement Sets** checks every `.gb`, `.gbc`, `.gba`, `.ws`, `.wsc`, `.lnx` and `.lyx` game on
the card while you watch: the game it's on, the systems done, sets downloaded and already on
the card, games RetroAchievements has no achievements for, and badges downloaded. Each game's
badges come straight after its set, so stopping with B leaves whole games.

Games whose set is already on the card are counted without asking the server. **X (Refresh
All)** asks again for every game, for sets that changed and for achievements earned elsewhere
since. Run it after adding games. Only ROM versions RetroAchievements knows can earn
achievements; for a version it doesn't support it sends a stand-in, which isn't counted.

A gold trophy on the shelf marks a game with a downloaded set, and **Achievements Ready**
shows with it for three seconds when that game starts.

## Wi-Fi and offline play

slotPlus uses Slot's Wi-Fi, which is up on the shelf and off while a game runs, and nothing is
fetched during play. So:

- Achievements earned in a game are saved on the card at once, survive ejects and power-offs,
  and are sent the next time Wi-Fi is up on the shelf. So are the games you played (with a
  downloaded set), so they show on your RetroAchievements profile. Network failures back off
  from 30 seconds to five minutes and retry on their own.
- A game with no set on the card earns nothing that session; download its set from Settings.
- The shelf's footer shows a small sync mark left of the clock while it sends, and an amber
  dot when something is waiting for the internet or needs attention. It is absent when the
  feature is off.
- `Logs/slot.log` has a `slot: achievements:` line for each set loaded (how many are active),
  each achievement earned, and each unlock and played game sent.

Keep `Saves/RetroAchievements` when backing up the card: it holds unsent unlocks and the
downloaded sets.

## How it works

Hashing, downloads, HTTPS and banner drawing run on background threads. The emulator copies
the game's memory after each frame into a bounded queue and never waits on the evaluator; if
evaluation falls behind, partial progress is reset rather than risking a false unlock, as it
is after a state load or rewind. Games on mGBA, gpSP, Gambatte, Cygne (WonderSwan) and the Lynx core are tracked, never over a
link cable.

```sh
cargo test -p slot-achievements -p slot-retro
```

The tests use a fake server and never send anything to a real account.
