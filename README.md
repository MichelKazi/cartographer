# Cartographer

It's like [Tactile](https://gitlab.com/lundal/tactile) but for macOS. I wanted it to work on my macbook with [Aerospace](https://github.com/nikitabobko/AeroSpace) as my WM.

Press a hotkey, get a grid overlay, press two keys to define a rectangle, window resizes to fit. That's it.

<!-- TODO: screenshots -->

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
```

Needs Rust and Xcode installed. Binary ends up in `target/release/cartographer`.

## Usage

```
./target/release/cartographer
```

Runs in the background with no dock icon. To quit:

```
pkill cartographer
```

## Built with

Rust + Swift. Rust does the heavy lifting (hotkey detection, grid logic, window management, Aerospace integration). Swift handles the overlay window because NSPanel needs to be subclassed to receive keyboard events without activating the app, and doing that from Rust would be miserable.

## License

MIT
