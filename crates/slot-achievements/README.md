# RetroAchievements

slotPlus earns **softcore RetroAchievements for Game Boy, Game Boy Color and Game Boy
Advance games**. Earning one shows a short banner in Slot's style, the achievement's title and
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

## Wi-Fi and offline play

slotPlus uses Slot's Wi-Fi, which is up on the shelf and off while a game runs. So:

- When a game starts whose achievement set isn't on the card yet, Wi-Fi stays up (or comes
  back) while that game's set and badges download, ahead of everything else, then goes off.
  It gives up after two minutes if the network never answers. Tracking starts the moment the
  set arrives, usually within seconds; anything earned before that can't be counted.
- While a game can't earn yet, a small red stop sign reading **RA** sits in the bottom-left
  corner of the screen. It goes as soon as the set is active, or when the server says the ROM
  has no achievements. If it stays, nothing in that session will count; go back to the shelf
  with Wi-Fi and let the set download.
- While the shelf sits idle with Wi-Fi up, slotPlus logs in and quietly prepares every
  `.gb`, `.gbc` and `.gba` game on the card (achievement sets and badges), a little at a time,
  so games start tracking straight away even with no network.
- Achievements earned in a game are saved on the card at once, survive ejects and power-offs,
  and are sent the next time Wi-Fi is up on the shelf. Network failures back off from 30
  seconds to five minutes and retry on their own.
- The shelf's footer shows a small sync mark left of the clock while it prepares or sends,
  with progress while caching, and an amber dot when something is waiting for the internet or
  needs attention. It is absent when the feature is off.

A newly added game needs its set downloaded once, so leave slotPlus on the shelf with Wi-Fi
for a while after adding games. Only ROM versions RetroAchievements knows can earn
achievements. Keep `Saves/RetroAchievements` when backing up the card: it holds unsent
unlocks and the prepared sets.

## How it works

Hashing, library preparation, HTTPS and banner drawing run on background threads, and
background work only runs while the shelf is idle. The emulator copies the game's memory
after each frame into a bounded queue and never waits on the evaluator; if evaluation falls
behind, partial progress is reset rather than risking a false unlock, as it is after a state
load or rewind. Games on mGBA, gpSP and Gambatte are tracked, never over a link cable.

```sh
cargo test -p slot-achievements -p slot-retro
```

The tests use a fake server and never send anything to a real account.
