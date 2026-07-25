# Alloy-Project

A 8x8 minecraft inspired sandbox written in rust and bevy using dead simple image driven modding...

## Dead Simple Modding

To add custom blocks without changing the code:
1. Place PNG images into the `assets/` folder.
2. Name them sequentially (`1.png`, `2.png`, etc.).
3. The game automatically detects them. Use the mouse scroll wheel in-game to select your blocks.

## Console Commands

If you run the game inside a terminal you have acces to commands :

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
