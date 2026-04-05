# Saddle Camera FPS Camera

Reusable first-person camera and locomotion-aware view-motion toolkit for Bevy.

The crate keeps gameplay-facing look state separate from cosmetic camera motion. It can run as a simple flat-ground internal controller for examples, prototypes, and debug scenes, or it can ingest externally authored motion from a separate controller while still handling bob, FOV, lean, recoil, shake, viewmodel lag, and comfort scaling.

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
        .add_plugins((DefaultPlugins, FpsCameraPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            Update,
        )))
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

## Public API

| Type | Purpose |
| --- | --- |
| `FpsCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `FpsCameraSystems` | Public ordering hooks: `ReadIntent`, `UpdateLocomotion`, `UpdateCameraState`, `ComposeEffects`, `SyncProjection`, `SyncTransform` |
| `FpsCamera` | Marker component for camera entities managed by this crate |
| `FpsCameraConfig` | Top-level tuning surface for look, locomotion-aware feedback, comfort, and effect layers |
| `FpsCameraIntent` | External intent inbox: move, mouse look, analog look, jump, sprint, crouch, aim, lean, and free-look |
| `FpsCameraRuntime` | Readable runtime state: logical position, velocity, yaw/pitch, stance alphas, trauma, bob phase, FOV, and composed render output |
| `FpsCameraExternalMotion` | Optional external locomotion seam for feeding authoritative position, velocity, grounded state, and landing impulses |
| `FpsCameraExternalEffects` | Optional additive extension seam for custom translation, rotation, or FOV effects |
| `FpsCameraCollisionFeedback` | Optional collision feedback component for external physics to push the camera away from walls |
| `CameraEffectLayer` / `CameraEffectStack` | Pure additive layer model for composing cosmetic view motion |
| Messages | `FootstepEvent`, `LandedEvent`, `CameraShakeRequest`, `CameraRecoilRequest` with optional per-request decay overrides |

## Configuration Overview

`FpsCameraConfig` is intentionally split by concern:

- `look`: mouse + analog look, clamp, inversion, smoothing
- `movement`: walk speed, sprint blend, acceleration, air control, gravity, eye height
- `crouch` / `jump`: stance transitions and designer-facing jump height
- `head_bob`, `tilt`, `landing`, `recoil`, `shake`, `collision`: additive presentation layers
- `viewmodel`: first-person weapon or hand lag driven from recent look and locomotion deltas
- `fov`, `aim`, `lean`, `free_look`: precision and tactical view control
- `comfort`: global weights for reducing motion without rewriting the pipeline

The default config targets a grounded exploration baseline. Arena, tactical, horror, and low-motion profiles are all reachable by parameter changes rather than code changes.

## Comfort Notes

The crate exposes comfort as weights instead of hard on/off branches. `ComfortConfig::low_motion()` is a good default for accessibility-sensitive experiences because it substantially reduces bob, roll, shake, landing compression, and dynamic FOV while preserving the underlying API. `ComfortConfig::vr_mode()` pushes those reductions further for camera stacks that need especially conservative motion.

## Integration Seams

- Internal locomotion is intentionally flat-ground and generic. It works well for prototypes, debug scenes, and showcase labs.
- For real character controllers or physics, feed `FpsCameraExternalMotion` every frame and let the camera stack derive presentation from that authoritative state.
- If another system needs custom view effects, write `FpsCameraExternalEffects` instead of mutating `Transform` directly.
- For camera collision avoidance, feed `FpsCameraCollisionFeedback` from your physics pipeline. The crate applies configurable push-back in `SyncTransform` without depending on any specific physics engine.
- `FpsCameraRuntime::viewmodel_translation` and `viewmodel_rotation` give a ready-made seam for weapon, hand, or cockpit meshes that should lag behind camera look.
- For dual-camera FPS setups, mirror `FpsCameraRuntime::visual_fov` into a view-model camera while keeping the main world camera managed here.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal look + move setup with defaults | `cargo run -p saddle-camera-fps-camera-example-basic` |
| `external_motion` | Character-controller bridge with support motion, landing feedback, and visible viewmodel sway | `cargo run -p saddle-camera-fps-camera-example-external-motion` |
| `grounded` | Heavier sprint, crouch, jump, and landing feedback | `cargo run -p saddle-camera-fps-camera-example-grounded` |
| `effects` | Timed recoil and trauma pulses | `cargo run -p saddle-camera-fps-camera-example-effects` |
| `tactical` | ADS, lean, and free-look oriented tuning | `cargo run -p saddle-camera-fps-camera-example-tactical` |
| `comfort` | Low-motion accessibility-focused tuning | `cargo run -p saddle-camera-fps-camera-example-comfort` |

Every example includes a live `saddle-pane` control surface so the main parameters can be tuned while the scene is running. Press **Tab** to toggle mouse capture and interact with the pane, or **Esc** to release the cursor.

The P0 FPS integration demo also ships with example-level smoke coverage:

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
