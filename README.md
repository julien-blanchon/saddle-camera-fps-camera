# Saddle Camera FPS Camera

Reusable first-person camera toolkit for Bevy with a minimal orientation core and opt-in motion/effects layers.

`FpsCameraPlugin` is now a pure camera module by default: it handles look intent, optional free-look, external motion ingestion, additive external camera effects, projection sync, and final transform sync. Internal walk/jump/crouch simulation, gait-driven bob, recoil, landing, footsteps, comfort scaling, and viewmodel lag all live behind optional plugins.

## Quick Start

```toml
[dependencies]
saddle-camera-fps-camera = { git = "https://github.com/julien-blanchon/saddle-camera-fps-camera" }
bevy = "0.18"
```

```rust,no_run
use bevy::prelude::*;
use saddle_camera_fps_camera::{FpsCamera, FpsCameraConfig, FpsCameraPlugin};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsCameraPlugin::new(OnEnter(DemoState::Gameplay), OnExit(DemoState::Gameplay), Update),
        ))
        .init_state::<DemoState>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Player Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.62, 6.0),
        FpsCamera,
        FpsCameraConfig::default(),
    ));
}
```

For examples and always-on tools, `FpsCameraPlugin::always_on(Update)` is the simple constructor.

If another controller owns authoritative movement, write `FpsCameraExternalMotion` every frame. If you want the old built-in locomotion + effects stack, use `FpsCameraLegacyPlugin` or opt into the granular plugins directly.

## Plugin Layers

| Plugin | Purpose |
| --- | --- |
| `FpsCameraPlugin` | Minimal core: look, free-look recentering, external motion sync, external additive effects, projection sync, transform sync |
| `FpsCameraLocomotionPlugin` | Optional internal flat-ground locomotion with sprint/crouch/jump state |
| `FpsCameraEffectsPlugin` | Optional bob, landing, recoil, trauma shake, dynamic FOV, lean, viewmodel lag, and footstep/landing messages |
| `FpsCameraLegacyPlugin` | Convenience wrapper that restores the pre-split full-feature stack |

## Public API

| Type | Purpose |
| --- | --- |
| `FpsCameraSystems` | Public ordering hooks: `ReadIntent`, `UpdateLocomotion`, `UpdateCameraState`, `ComposeEffects`, `SyncProjection`, `SyncTransform` |
| `FpsCamera` | Marker component for camera entities managed by this crate |
| `FpsCameraConfig` | Unified tuning surface. Core uses look/aim/free-look/base FOV/collision, optional plugins consume locomotion and presentation fields |
| `FpsCameraIntent` | External intent inbox. Core consumes look + aim/free-look input; optional plugins may also consume move/jump/sprint/crouch/lean |
| `FpsCameraRuntime` | Readable runtime state: logical position, velocity, yaw/pitch, stance alphas, trauma, bob phase, FOV, and composed render output |
| `FpsCameraExternalMotion` | Authoritative movement seam for position, velocity, grounded state, landing impulse, and optional eye-height/stance overrides |
| `FpsCameraExternalEffects` | Optional additive extension seam for custom translation, rotation, or FOV effects |
| `FpsCameraCollisionFeedback` | Optional collision feedback component for external physics to push the camera away from walls |
| `CameraEffectLayer` / `CameraEffectStack` | Pure additive layer model for composing cosmetic view motion |
| Messages | `FootstepEvent`, `LandedEvent`, `CameraShakeRequest`, `CameraRecoilRequest` are registered by `FpsCameraEffectsPlugin` / `FpsCameraLegacyPlugin` |

## Configuration Overview

`FpsCameraConfig` is intentionally split by concern:

- Core: `look`, `aim`, `free_look`, `fov.base_fov`, `collision`
- Optional locomotion: `movement`, `crouch`, `jump`
- Optional presentation: `head_bob`, `tilt`, `landing`, `recoil`, `shake`, `viewmodel`, `comfort`
- Shared by optional layers: `fov.speed_boost`, `fov.sprint_boost`, `lean`

This keeps the public config flat for legacy/full-feature users while making the default runtime a pure camera-orientation stack.

## Integration Seams

- `FpsCameraPlugin` alone gives you a stable first-person orientation core that can stay fixed at its spawn transform or follow `FpsCameraExternalMotion`.
- For real character controllers or physics, feed `FpsCameraExternalMotion` every frame and let optional plugins derive presentation from that authoritative state.
- If another system needs custom view effects, write `FpsCameraExternalEffects` instead of mutating `Transform` directly.
- For camera collision avoidance, feed `FpsCameraCollisionFeedback` from your physics pipeline. The crate applies push-back in `SyncTransform` without depending on any specific physics engine.
- `FpsCameraRuntime::viewmodel_translation` and `viewmodel_rotation` are only populated when `FpsCameraEffectsPlugin` (or `FpsCameraLegacyPlugin`) is active.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Legacy convenience wrapper with built-in walk/jump/crouch defaults | `cargo run -p saddle-camera-fps-camera-example-basic` |
| `external_motion` | Minimal core + optional effects layered on top of an authoritative controller | `cargo run -p saddle-camera-fps-camera-example-external-motion` |
| `grounded` | Legacy wrapper with heavier sprint, crouch, jump, and landing feedback | `cargo run -p saddle-camera-fps-camera-example-grounded` |
| `effects` | Legacy wrapper with timed recoil and trauma pulses | `cargo run -p saddle-camera-fps-camera-example-effects` |
| `tactical` | Legacy wrapper tuned for ADS, lean, and free-look | `cargo run -p saddle-camera-fps-camera-example-tactical` |
| `comfort` | Legacy wrapper with low-motion accessibility-focused tuning | `cargo run -p saddle-camera-fps-camera-example-comfort` |

Every example includes a live `saddle-pane` control surface so the main parameters can be tuned while the scene is running. Press **Tab** to toggle mouse capture and interact with the pane, or **Esc** to release the cursor.

The P0 external-motion integration demo also ships with example-level smoke coverage:

```bash
cargo run -p saddle-camera-fps-camera-example-external-motion --features e2e -- fps_external_motion_smoke
```

## Workspace Lab

The richer lab app lives inside the crate at `shared/camera/saddle-camera-fps-camera/examples/lab`:

```bash
cargo run -p saddle-camera-fps-camera-lab
```

With E2E enabled:

```bash
cargo run -p saddle-camera-fps-camera-lab --features e2e -- fps_camera_smoke
```

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
