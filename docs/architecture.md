# Architecture

## Layered Runtime

The crate now has an explicit plugin split:

1. `FpsCameraPlugin`
   Core look/orientation runtime.
   Reads `FpsCameraIntent`, applies yaw/pitch and optional free-look offsets, copies `FpsCameraExternalMotion`, composes `FpsCameraExternalEffects`, syncs projection, then writes the final camera transform.
2. `FpsCameraLocomotionPlugin`
   Optional internal flat-ground motion model.
   Consumes move/sprint/crouch/jump intent and advances the old built-in locomotion state when no external motion source is active.
3. `FpsCameraEffectsPlugin`
   Optional presentation/gameplay layer.
   Consumes runtime motion data to derive bob, landing compression, recoil, trauma shake, lean, viewmodel lag, and landing/footstep messages.
4. `FpsCameraLegacyPlugin`
   Convenience wrapper that enables all three layers together.

This keeps the default crate entry point a pure camera module while preserving the old full-feature behavior as an explicit opt-in.

## Intent To Camera Flow

The public system sets still execute in the same high-level order:

1. `ReadIntent`
   `FpsCameraIntent` is resolved into yaw/pitch deltas, aim sensitivity scaling, and free-look offsets.
2. `UpdateLocomotion`
   The core copies `FpsCameraExternalMotion` when present.
   The optional locomotion plugin may instead advance the internal walk/jump/crouch model.
3. `UpdateCameraState`
   The core recenters free-look offsets.
   The optional effects plugin derives gait, landing, trauma decay, tilt, lean, and viewmodel state.
4. `ComposeEffects`
   The core composes free-look plus `FpsCameraExternalEffects`.
   The optional effects plugin can replace that with the richer bob/recoil/landing stack.
5. `SyncProjection`
   The resolved FOV is written into Bevy’s `PerspectiveProjection`.
6. `SyncTransform`
   The final render transform is written in `PostUpdate` before transform propagation.

## Logical Vs Render Camera

`FpsCameraRuntime::position`, `yaw`, and `pitch` remain the logical state.

Cosmetic motion never mutates that logical look directly. Optional view motion is composed into `FpsCameraRuntime::effect_stack`, then added to the logical state to produce:

- `render_translation`
- `render_rotation`
- `visual_fov`

That separation keeps gameplay-facing state readable and stable even when presentation becomes aggressive.

The same split applies to first-person props. `FpsCameraRuntime::viewmodel_translation` and `viewmodel_rotation` are derived presentation outputs populated by the optional effects plugin; they stay zero in the core-only runtime.

## External Motion Seam

`FpsCameraExternalMotion` is the authoritative seam for external character controllers and physics.

It now carries:

- `position`
- `velocity`
- `grounded`
- `landing_impulse`
- optional `eye_height`
- optional `crouch_alpha`
- optional `sprint_alpha`

The important distinction is that the core camera no longer simulates gameplay motion by default. If your game already owns locomotion, feed that state directly here and let optional camera layers derive presentation from it.

## Internal Locomotion Model

The built-in locomotion path still exists, but only behind `FpsCameraLocomotionPlugin` / `FpsCameraLegacyPlugin`.

It is deliberately simple:

- flat ground anchored from the spawn transform
- camera-relative planar intent
- acceleration / deceleration shaping
- sprint and crouch speed scaling
- jump height derived from gravity and desired apex
- grounded / airborne distinction for gravity and air control

This remains useful for examples, tooling, and debug scenes, but it is no longer the default runtime behavior.

## Effect Layer Composition

With `FpsCameraEffectsPlugin`, each motion effect becomes a `CameraEffectLayer`:

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

The crate still exposes both `CameraEffectLayer` and `compose_effect_stack()` so consumers can extend the additive model without reaching into internal systems.

## Extension Guidance

Prefer these seams in order:

1. Feed `FpsCameraIntent` if you want to drive stock look behavior from a different input source.
2. Feed `FpsCameraExternalMotion` if a separate controller owns movement and grounding.
3. Add `FpsCameraLocomotionPlugin` only if you explicitly want the built-in flat-ground motion model.
4. Add `FpsCameraEffectsPlugin` only if you explicitly want bob/recoil/landing/viewmodel behavior and message-driven presentation.
5. Feed `FpsCameraExternalEffects` if another system wants custom camera punch, scripted sway, breathing, or custom FOV changes.
6. Feed `FpsCameraCollisionFeedback` if a physics system detects camera-to-wall collisions.

Avoid writing `Transform` directly from outside the crate. That bypasses the logical/cosmetic split and makes ordering brittle.
