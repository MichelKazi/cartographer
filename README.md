# Cartographer

It's like [Tactile](https://gitlab.com/lundal/tactile) but for macOS. I wanted it to work on my macbook with [Aerospace](https://github.com/nikitabobko/AeroSpace) as my WM.

Press a hotkey, get a grid overlay, press two keys to define a rectangle, window resizes to fit. That's it.

> **Note:** I try and be mindful of pushing too much AI code as open source. Claude Code helped me finally clean up and finish this project. This shit is the future.

> **Alpha software.** This will break. I may introduce breaking changes at any time as I go. Use at your own risk.



https://github.com/user-attachments/assets/94b19798-cc1f-48c4-bf9d-f3694793fcc6


## How it works

1. Press `alt+cmd+t` to show the grid overlay
2. Press a grid key to select the first corner (it highlights)
3. Press another grid key to select the second corner
4. Window resizes to the bounding rectangle of those two cells
5. Overlay disappears

```
┌─────┬─────┬─────┬─────┐
│  Q  │  W  │  E  │  R  │
├─────┼─────┼─────┼─────┤
│  A  │  S  │  D  │  F  │
├─────┼─────┼─────┼─────┤
│  Z  │  X  │  C  │  V  │
└─────┴─────┴─────┴─────┘
```

Same key twice = single cell tile. `Escape` dismisses. Pressing the hotkey again while the overlay is up also dismisses it.

If you wait more than 1 second after the first key, it resets.

## Aerospace

If [Aerospace](https://github.com/nikitabobko/AeroSpace) is running, Cartographer uses its CLI (`aerospace resize`) to adjust split ratios in the tiling tree instead of fighting the WM with direct accessibility API calls. This means your windows stay tiled and Aerospace doesn't snap them back.

One axis might not resize if there's no sibling in that direction. That's an Aerospace thing, not a bug.

## Install

### Homebrew

```
brew install michelkazi/tap/cartographer
```

### curl

```
curl -sSfL https://raw.githubusercontent.com/michelkazi/cartographer/main/install.sh | sh
```

Apple Silicon only for now. Requires macOS and Accessibility permission (it'll prompt you on first run).

### From source

If you'd rather build it yourself (or you're on Intel):

```
git clone https://github.com/michelkazi/cartographer.git
cd cartographer
cargo build --release
cp target/release/cartographer /usr/local/bin/
cp com.michelkazi.cartographer.plist ~/Library/LaunchAgents/
```

Needs Rust and Xcode installed. The plist gives you launch at login.

## Usage

```
cartographer
```

Runs in the background with a menu bar icon (grid icon). Quit from the menu bar or:

```
pkill cartographer
```

### Launch at login

If you installed via Homebrew:

```
brew services start cartographer
```

The curl installer and from-source instructions set up launch at login automatically.

## Roadmap

what's done and what I want to get to eventually

- [x] grid overlay with two-key window tiling
- [x] aerospace integration (resize via CLI)
- [x] hotkey toggle (alt+cmd+t shows/hides)
- [x] selection timeout (1s reset)
- [ ] config file (TOML probably) for hotkey, grid size, colors, key bindings
- [ ] configurable grid dimensions (not just 4x3)
- [ ] custom key layout (maybe you don't want QWER/ASDF/ZXCV)
- [ ] custom colorway (overlay tint, highlight, label color)
- [ ] multi-monitor support (show overlay on the focused window's screen)
- [x] menu bar icon with quit/preferences
- [x] launch at login
- [ ] animation on show/hide (maybe, if it doesn't feel slow)
- [ ] intel build / universal binary

no promises on timelines, I work on this when I feel like it

## Built with

Rust + Swift. Rust does the heavy lifting (hotkey detection, grid logic, window management, Aerospace integration). Swift handles the overlay window because NSPanel needs to be subclassed to receive keyboard events without activating the app, and doing that from Rust would be miserable.

## License

MIT
