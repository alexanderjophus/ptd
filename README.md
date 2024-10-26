# ptd

Started life as a pokemon tower defense game. Now it's just a tower defense game.

## Getting Started

Clone (and make sure you've clone the lfs files too (glb et al)).

Then

```bash
cargo run
```

There's a feature flag for debug `--features debug` which will show the grid and other helpful things.

## Controls

- `WASD` to move the camera
- Arrow keys to move the towers
- `Space` to place a tower
- `T` to toggle tower choice
- `Enter` to start the wave

## Features

- Grid based system
- Towers are obstacles the enemy must be able to navigate around

### Todo list

(Not in any particular order, and this may become outdated)

- [ ] Towers have a base shape that is a hexomino based on a cube net. (i.e. unravel a cube into a 2D shape)
- [ ] Pathfinding is visible during placement phase
- [ ] Moddable towers/enemies. i.e. a ron file that points to new valid glb files, with all the towers config done.
- [ ] Pathfinding 1.1 (disable blocking a path)
- [ ] End of wave logic (currently crashes trying to go back to placement phase)
- [ ] More tower types
- [ ] More enemy types

## Credits

Check [credits](assets/credits.md) for more details.
