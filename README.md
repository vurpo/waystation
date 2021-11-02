# WayStation

This is not beta software, or experimental quality software, this is proof-of-concept quality software (read: hacked-up code that barely works well enough to demostrate my idea).

Based on the Anvil compositor from the [Smithay](https://smithay.github.io) project.

## What?

This is a Wayland compositor intended for devices which only have a gamepad/joystick as their input device, and want to remain relatively low-power. Intended for running any game that can run as a Wayland client, for example emulators like RetroArch, or games like SuperTuxKart.

Not intended for running "traditional" desktop GUI apps (like GTK and stuff), after all the compositor is intended for devices without a keyboard or mouse.

## Why?

These kinds of devices (for example, the many RK3326 handheld gaming devices that exist such as the Odroid Go Super, or homemade Raspberry Pi game machines) usually run EmulationStation as their main user interface. ES works without a display server and accesses the screen directly using DRM/KMS, and when launching a game it releases the screen and expects the launched game to grab the screen for itself in the same way (often supported via libraries such as SDL).

This *mostly* works but is not a very nice user experience, as there is no standardized way to exit the running game using the gamepad (each game must implement some way to exit it), no way to temporarily return to the ES menu and leave the game running at the same time, no way to render an on-screen display e.g. while setting the volume or if the battery is low, etc.

I wanted to experiment with implementing a system where you can still launch games via a gamepad menu like before, but the UI would always remain accessible even while a game is running, because of the above issues. In theory, this compositor should be compatible with any game that can launch as a Wayland client, which should be most of them.

## Building

Packages needed before building (on Fedora):

`libudev-devel mesa-libgbm-devel libxcb-devel libxkbcommon-devel libinput-devel`