# minecraft.rs

## A low-ish rust minecraft launcher

[![forthebadge](https://forthebadge.com/images/badges/0-percent-optimized.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/60-percent-of-the-time-works-every-time.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/contains-tasty-spaghetti-code.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/mom-made-pizza-rolls.svg)](https://forthebadge.com)
[![wakatime](https://wakatime.com/badge/github/glowsquid-launcher/minecraft-rs.svg?style=for-the-badge)](https://wakatime.com/badge/github/glowsquid-launcher/minecraft-rs)

This is meant to be a low-level structural launcher where everything is done separately.
You don't launch and let it download the assets. You first download the assets and then launch.
This is meant to be used as a library, but a standalone is also in the works (mainly for testing)

**MICROSOFT ONLY** becauce mojang is being removed in the future. Migrate now

## Testing

Testing basically works by:

- run something in minecraft CLI
- see if it makes sense

What makes sense should make sense for the launcher
