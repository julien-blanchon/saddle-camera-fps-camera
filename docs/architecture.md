# Architecture

## Intent To Camera Flow

The runtime is intentionally layered:

1. `FpsCameraIntent` collects raw user or AI intent.
2. `ReadIntent` resolves mouse and analog look into yaw/pitch or free-look offsets.
3. `UpdateLocomotion` advances either the internal flat-ground locomotion state or copies `FpsCameraExternalMotion`.
4. `UpdateCameraState` derives stance alphas, bob phase, landing impulses, trauma decay, viewmodel lag state, and runtime diagnostics.
5. `ComposeEffects` builds additive effect layers and resolves final render-space translation, rotation, and FOV.
6. `SyncProjection` writes the resolved world FOV into Bevy’s `Projection`.
7. `SyncTransform` writes the final render transform in `PostUpdate` before transform propagation.

## Logical Vs Render Camera

`FpsCameraRuntime::position`, `yaw`, and `pitch` are the logical state.

Cosmetic motion never mutates that logical look directly. Bob, landing compression, lean, recoil, shake, and external custom layers are composed into `FpsCameraRuntime::effect_stack`, then added to the logical state to produce:

- `render_translation`
- `render_rotation`
- `visual_fov`

That separation keeps gameplay-facing state readable and stable even when presentation becomes aggressive.

The same split now applies to first-person props. `FpsCameraRuntime::viewmodel_translation` and `viewmodel_rotation` are derived presentation outputs that can drive weapon or hand meshes without polluting the gameplay-facing yaw, pitch, or world transform.

## Locomotion Model

The built-in locomotion path is deliberately simple:

- flat ground anchored from the spawn transform
- camera-relative planar intent
- acceleration / deceleration shaping
- sprint and crouch speed scaling
- jump height derived from gravity and desired apex
- grounded / airborne distinction for gravity and air control

This is enough for examples, tooling, and low-dependency prototypes. Games with their own movement stack should treat `FpsCameraExternalMotion` as the authoritative seam.

When an external controller owns landing detection, it can either feed `landing_impulse` through `FpsCameraExternalMotion` or send explicit `CameraShakeRequest` / `CameraRecoilRequest` messages with per-event duration overrides.

## Effect Layer Composition

Each effect becomes a `CameraEffectLayer`:

```text
base logical pose
+ head bob / idle sway
+ landing compression
+ trauma shake
+ lean
+ tilt + recoil + free-look offsets
+ external custom effects
= final render pose
```

The crate exposes both `CameraEffectLayer` and `compose_effect_stack()` so consumers can reason about or extend the additive model without reaching into the internal systems.

Viewmodel lag is derived alongside the effect stack, but it is kept separate because it usually drives a child mesh hierarchy instead of the world camera transform.

## Projection Path

The world FOV target is derived from:

- base FOV
- speed-based boost
- sprint boost
- ADS multiplier
- external custom FOV deltas

That target is smoothed into `FpsCameraRuntime::visual_fov`, then `SyncProjection` writes it into the active `PerspectiveProjection`.

## Extending The Stack

For extension, prefer these seams in order:

1. Feed `FpsCameraIntent` if you want to drive the stock logic with a different input source.
2. Feed `FpsCameraExternalMotion` if a separate controller owns movement and grounding.
3. Feed `FpsCameraExternalEffects` if another system wants to add camera punch, scripted sway, breathing, or other custom presentation layers.
4. Read `FpsCameraRuntime::viewmodel_translation` / `viewmodel_rotation` if a separate system owns weapon or cockpit presentation.

Avoid writing `Transform` directly from outside the crate. That bypasses the logical/cosmetic split and makes ordering brittle.
