# `saddle-camera-fps-camera-lab`

Crate-local showcase and verification app for `saddle-camera-fps-camera`.

## Run

```bash
cargo run -p saddle-camera-fps-camera-lab
```

## E2E

```bash
cargo run -p saddle-camera-fps-camera-lab --features e2e -- fps_camera_smoke
```

Available scenarios:

- `fps_camera_smoke`
- `fps_camera_look`
- `fps_camera_movement`
- `fps_camera_effects`
- `fps_camera_comfort`
- `fps_camera_viewmodel`

## BRP

```bash
cargo run -p saddle-camera-fps-camera-lab
uv run --project .codex/skills/bevy-brp/script brp world query \
  bevy_ecs::name::Name saddle_camera_fps_camera::components::FpsCameraRuntime
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle-camera-fps-camera-lab.png
```

Use the reflected component path reported by BRP, not the crate-root re-export name.
