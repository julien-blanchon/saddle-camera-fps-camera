# Configuration

All values use Bevy conventions: distances are in world units, rotations are in radians, and FOV values are perspective radians.

## `FpsCameraConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `look` | `LookConfig` | default | n/a | Mouse and analog look behavior |
| `movement` | `MovementConfig` | default | n/a | Baseline locomotion and eye height |
| `crouch` | `CrouchConfig` | default | n/a | Crouch stance height, speed, and smoothing |
| `jump` | `JumpConfig` | default | n/a | Jump apex and fall behavior |
| `head_bob` | `HeadBobConfig` | default | n/a | Gait-linked bob and idle sway |
| `fov` | `FovConfig` | default | n/a | Base FOV and motion-driven boosts |
| `shake` | `ShakeConfig` | default | n/a | Trauma shake envelope |
| `tilt` | `TiltConfig` | default | n/a | Strafe roll |
| `landing` | `LandingImpactConfig` | default | n/a | Landing compression |
| `recoil` | `RecoilConfig` | default | n/a | Additive recoil recovery |
| `viewmodel` | `ViewmodelLagConfig` | default | n/a | First-person weapon or hand lag driven by recent motion |
| `aim` | `AimConfig` | default | n/a | ADS transition and precision scaling |
| `lean` | `LeanConfig` | default | n/a | Tactical lean angle and offset |
| `free_look` | `FreeLookConfig` | default | n/a | Temporary camera-only yaw/pitch offsets |
| `comfort` | `ComfortConfig` | default | n/a | Global motion reduction weights |

## `LookConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `sensitivity` | `Vec2` | `(0.0022, 0.0020)` | `0.0015..0.004` | Mouse radians per delta unit for yaw and pitch |
| `invert_x` / `invert_y` | `bool` | `false` | `false/true` | Axis inversion |
| `smoothing` | `DecayConfig` | off | `0..20` decay | Smoother mouse / stick feel. Higher values feel snappier |
| `pitch_min` / `pitch_max` | `f32` | `-1.50 / 1.50` | keep away from `±FRAC_PI_2` | Prevents degenerate up-vector and inverted yaw behavior |
| `analog` | `AnalogLookConfig` | default | n/a | Right-stick look curve |

Tradeoff: more smoothing softens jitter and controller noise, but it also reduces the crispness expected in precision shooters.

## `AnalogLookConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables right-stick look |
| `max_radians_per_second` | `Vec2` | `(3.6, 2.8)` | `1.5..6.0` | Maximum turn rate at full stick |
| `deadzone` | `f32` | `0.18` | `0.08..0.25` | Filters small stick noise |
| `outer_deadzone` | `f32` | `0.05` | `0.0..0.15` | Normalizes worn controllers near full deflection |
| `exponent` | `f32` | `1.35` | `1.0..2.0` | Higher values give finer center precision and slower initial turn |

## `MovementConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `eye_height` | `f32` | `1.62` | `1.4..1.8` | Standing eye height above logical ground |
| `walk_speed` | `f32` | `4.8` | `3.0..8.0` | Base grounded movement speed |
| `sprint_multiplier` | `f32` | `1.45` | `1.1..2.0` | Multiplies walk speed while sprinting |
| `sprint_transition` | `DecayConfig` | `12.0` decay | `4..20` | How quickly sprint alpha blends in and out |
| `acceleration` | `f32` | `30.0` | `8.0..40.0` | Ground responsiveness |
| `deceleration` | `f32` | `34.0` | `8.0..45.0` | Stop responsiveness |
| `air_acceleration` | `f32` | `10.0` | `2.0..20.0` | Air steering |
| `air_control` | `f32` | `0.55` | `0.1..1.0` | Scales desired air velocity |
| `max_air_speed` | `f32` | `4.0` | `1.0..8.0` | Caps internally simulated air movement |
| `gravity` | `f32` | `22.0` | `10.0..35.0` | Stronger values feel heavier |
| `terminal_velocity` | `f32` | `45.0` | `20.0..80.0` | Limits falling speed |

## `CrouchConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables crouch state |
| `eye_height` | `f32` | `1.1` | `0.8..1.3` | Crouched eye height |
| `speed_multiplier` | `f32` | `0.58` | `0.3..0.8` | Crouched move speed |
| `transition` | `DecayConfig` | `14.0` decay | `6..25` | Higher values feel snappier |

## `JumpConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables internal jump simulation |
| `height` | `f32` | `1.2` | `0.5..2.0` | Designer-facing jump apex |
| `fall_multiplier` | `f32` | `1.15` | `1.0..2.0` | Faster descents add weight |
| `landing_velocity_threshold` | `f32` | `2.5` | `1.0..6.0` | Minimum impact speed before landing feedback fires |

## `HeadBobConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables gait-linked bob |
| `amplitude` | `Vec3` | `(0.025, 0.045, 0.018)` | low single-digit centimeters | Translation amplitude for lateral, vertical, and fore-aft bob |
| `stride_length` | `f32` | `1.55` | `1.0..2.5` | Lower values increase cadence |
| `sprint_multiplier` | `f32` | `1.35` | `1.0..2.0` | Boosts bob intensity while sprinting |
| `crouch_multiplier` | `f32` | `0.65` | `0.2..1.0` | Reduces bob while crouched |
| `idle_sway_translation` | `Vec3` | small | very small | Subtle breathing-style drift when nearly stationary |
| `idle_sway_rotation` | `Vec2` | small | very small | Idle pitch and roll micro-motion |
| `idle_sway_frequency` | `f32` | `1.3` | `0.5..2.0` | Breathing cadence |

Tradeoff: stronger bob sells speed and weight, but it raises motion-sickness risk. Lower `comfort.bob_weight` first before rewriting amplitudes for accessibility.

## `FovConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `base_fov` | `f32` | `85°` | `70°..110°` | Neutral world FOV |
| `speed_boost` | `f32` | `8°` | `0°..15°` | Scales with speed ratio |
| `sprint_boost` | `f32` | `3°` | `0°..8°` | Additional sprint-only push |
| `response` | `DecayConfig` | `10.0` decay | `4..20` | Higher values feel snappier |

## `ShakeConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `translation_amplitude` | `Vec3` | `(0.03, 0.04, 0.02)` | small | Positional shake limits |
| `rotation_amplitude` | `Vec3` | `(0.03, 0.04, 0.02)` | small | Pitch/yaw/roll shake limits |
| `decay_rate` | `f32` | `1.85` | `0.8..4.0` | Trauma drain rate |
| `frequency` | `f32` | `27.0` | `8..40` | Noise frequency |
| `noise_profile` | `ShakeNoiseProfile` | `Standard` | `Standard`, `Handheld`, `Explosion`, `Rumble` | Picks the procedural shake signature while still using the same trauma envelope |
| `max_trauma` | `f32` | `1.0` | `0.5..1.5` | Clamp for injected trauma |
| `seed` | `f32` | `0.37` | any | Deterministic phase offset for tests and replays |

`CameraShakeRequest::duration_override` can temporarily replace the global decay rate for one event without mutating `ShakeConfig`.

## `TiltConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables strafe roll |
| `max_roll` | `f32` | `4.5°` | `0°..8°` | Larger values feel more arcadey |
| `response` | `DecayConfig` | `16.0` decay | `8..24` | Return speed |

## `LandingImpactConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables landing compression |
| `translation_amount` | `f32` | `0.14` | `0.02..0.25` | Downward dip magnitude |
| `pitch_amount` | `f32` | `7°` | `0°..12°` | Pitch-down impact |
| `max_impulse` | `f32` | `1.0` | `0.2..2.0` | Landing impulse clamp |
| `response` | `DecayConfig` | `10.0` decay | `4..20` | Recovery speed |

## `RecoilConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables visual recoil |
| `recovery` | `DecayConfig` | `18.0` decay | `8..30` | Return-to-neutral speed |
| `max_pitch` | `f32` | `14°` | `2°..20°` | Clamp for stacked pitch recoil |
| `max_yaw` | `f32` | `9°` | `1°..15°` | Clamp for stacked yaw recoil |

`CameraRecoilRequest::duration_override` can temporarily replace `recovery` for one burst, which is useful for differentiating sidearms, rifles, and cinematic one-off kicks.

## `ViewmodelLagConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables viewmodel lag output |
| `translation_scale` | `Vec3` | small | `0.0..0.08` | Maps recent look deltas into weapon or hand translation |
| `rotation_scale` | `Vec3` | small | `0.0..0.6` | Maps recent look deltas into local pitch, yaw, and roll |
| `movement_scale` | `Vec3` | small | `0.0..0.08` | Adds velocity-driven lag so sprinting or strafing can sway the viewmodel |
| `response` | `DecayConfig` | `14.0` decay | `6..30` | Controls how quickly the lag catches up to the camera |
| `max_translation` | `Vec3` | small | `0.01..0.18` | Clamp for local viewmodel translation |
| `max_rotation` | `Vec3` | small | `0.02..0.45` | Clamp for local viewmodel rotation |

Use `FpsCameraRuntime::viewmodel_translation` and `viewmodel_rotation` to drive a separate mesh rig or child hierarchy. The crate intentionally does not assume a specific weapon, hands, or cockpit layout.

## `AimConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables ADS blend |
| `transition` | `DecayConfig` | `16.0` decay | `6..30` | ADS in/out speed |
| `sensitivity_scale` | `f32` | `0.65` | `0.2..1.0` | Precision multiplier while aiming |
| `fov_multiplier` | `f32` | `0.84` | `0.6..1.0` | World FOV compression while aiming |

## `LeanConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables lean |
| `max_angle` | `f32` | `10°` | `0°..18°` | Roll angle when fully leaned |
| `lateral_offset` | `f32` | `0.09` | `0.0..0.2` | Sideways camera shift |
| `response` | `DecayConfig` | `15.0` decay | `6..25` | Lean speed and return |

## `FreeLookConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `false/true` | Enables temporary camera-only offsets |
| `yaw_limit` | `f32` | `55°` | `10°..90°` | Maximum camera-only yaw |
| `pitch_limit` | `f32` | `18°` | `5°..35°` | Maximum camera-only pitch |
| `recenter` | `DecayConfig` | `12.0` decay | `4..20` | Return speed after free-look ends |

## `ComfortConfig`

| Field | Type | Default | Recommended Range | Effect |
| --- | --- | --- | --- | --- |
| `bob_weight` | `f32` | `1.0` | `0.0..1.0` | Multiplies head bob and idle sway |
| `roll_weight` | `f32` | `1.0` | `0.0..1.0` | Multiplies tilt and lean roll |
| `shake_weight` | `f32` | `1.0` | `0.0..1.0` | Multiplies trauma shake |
| `dynamic_fov_weight` | `f32` | `1.0` | `0.0..1.0` | Multiplies speed/sprint FOV gain |
| `landing_weight` | `f32` | `1.0` | `0.0..1.0` | Multiplies landing compression |

`ComfortConfig::low_motion()` is the recommended accessibility preset.
`ComfortConfig::vr_mode()` is the most conservative preset and is intended for especially motion-sensitive camera stacks.
