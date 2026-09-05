# Blender bear and MCP setup

The editable recreation is `assets/models/toybox_bear.blend`; the toy-only
export is `assets/models/toybox_bear.glb`. The studio render is
`docs/verification/blender_bear.png`. The game continues to use its procedural
renderer; these are authoring assets, not a runtime replacement.

The bear follows `src/toys/bear.rs` and the face in `src/toys/primitives.rs`,
with 27 named parts, eight-segment spheres, flat shading, the cream bow, and
the cherry palette with slot 0 / color index 0 offsets. Blender's lighting
and AgX color management affect the rendered appearance. Positions map from
game `(x, y, z)` to Blender `(x, z, y + 0.26)`.

## Installed on this machine

- Blender: `C:\Program Files\Blender Foundation\Blender 5.0\blender.exe`
- Community MCP server: `blender-mcp` 1.9.1, installed using uv 0.12.10.
- Codex server name: `blender`, using the absolute executable
  `C:\Users\Kalai\.local\bin\blender-mcp.exe`.
- Environment: `BLENDER_HOST=127.0.0.1`, `BLENDER_PORT=9876`,
  `DISABLE_TELEMETRY=true`.
- Blender add-on: `blender_mcp.py` in the user's Blender 5.0 add-ons folder;
  enabled and saved in preferences, with telemetry consent disabled.

The add-on and server completed an MCP handshake (protocol 5), listed tools,
and successfully executed the modeling script in Blender 5.0.0. A separate
Blender window was opened so the previously running instance was preserved.
The installer's first console output encountered a Windows encoding error;
the installed add-on was subsequently verified by loading and connecting it.

If Codex does not show the new Blender tools yet, restart Codex to reload
its MCP configuration. In Blender, the 3D viewport sidebar's **MCP for
Blender** tab provides **Connect to MCP server**. Only one Blender instance
can own port 9876 at a time.

## Recreate and render

Execute `scripts/blender_bear.py` using the MCP `execute_blender_code` tool.
Set `TOYBOX_ROOT` in that execution namespace if the checkout moves. Each
execution creates a separate scene; it saves the named output files above.
The script exports only the toy collection to GLB, retaining the studio in
the Blender file. To render the exact preview filename, execute this in
Blender after building the scene:

```python
bpy.ops.render.render(write_still=True)
```

Setup references: [Blender MCP](https://github.com/ahujasid/blender-mcp) and
[Codex MCP configuration](https://developers.openai.com/codex/mcp).
