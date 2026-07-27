# Alloy-Project

<img width="1330" height="807" alt="image" src="https://github.com/user-attachments/assets/82a3fe38-0540-4485-96d5-ec3ee8155e1d" />


A 8x8 minecraft inspired sandbox written in rust and bevy using dead simple image driven modding...

## Default Game

The base game is bare bone by default (juste the gride block 0 is added by default) because your are meant to add blocks yourself via the help of assetpacks (.zip containing a list of premade block)

## Assetpacks

I have already made an assetpack for ap you can consider it as a "official" blocks pack for the game. You can download it in the release tab.

## Dead Simple Modding

To add custom blocks without changing the code:
1. Place PNG images into the `assets/` folder.
2. Name them how you want (`stone.png`, `sand.png`, etc.).
3. The game automatically detects them and add them. Use the mouse scroll wheel in-game to select your blocks.

stone.png (image) -> stone (block ingame) 

## Console Commands

If you run the game inside a terminal you have acces to commands

(this will be implemented directly into the game later.) :

- `/fly` - Enable flight (Space to go up, Shift to go down)
- `/fall` - Disable flight and gravity
- `/noclip` - Toggle noclip mode
- `/tp <x> <y> <z>` - Teleport to coordinates
- `/spawn` - Teleport to spawn point
- `/reset` - Reset the world and player position
- `/save [name]` - Save the world to the `grids/` folder
- `/load [name]` - Load a saved world
- `/list` - List all saved worlds
- `/sensivity <value>` - Adjust mouse sensitivity
- `/speed <value>` - Adjust movement speed
- `/fov <value>` - Change camera field of view
- `/stop` - Quit the game
- `/help` - Show all available commands

## The Grid System (save/load/list)

The world uses a sparse grid (HashMap) instead of a heavy 3D array. Each block is stored using its exact 3D coordinates and linked to a game entity. This means only placed blocks use memory, performance stays high, and saving or loading the world to a text file is simple.

## Thanks to Rust and Bevy
<img width="482" height="240" alt="image" src="https://github.com/user-attachments/assets/d16f6906-784e-4bcb-86a9-ada6606abcba" />

<img width="561" height="143" alt="image" src="https://github.com/user-attachments/assets/ff90517d-ebad-4624-84e1-dbad6e357e17" />

