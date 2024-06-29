# LuckyWM

<img src=".extras/showcase-one.png" height="432">

Minimal, flexible and customizable, window manager for X.

### Note

Lucky is in heavy development and not fully suited for use, new features are being added
in a fast pace.

### Features and Roadmap

- [x] Multi-monitor support
- [x] Tiling layout
- [x] Multiple desktops
- [x] Window focus follows mouse
- [x] Custom keybinds for actions and commands
- [x] Customizable decorations
- [x] Startup programs and commands
- [ ] Fullscreen and minimize screens
- [x] Status bar support
- [ ] Titles
- [ ] Floating layout
- [ ] Interactive Resizing
- [ ] Compositor support
- [ ] Gaps
- [ ] ICCCM compliance
- [w] Full EWMH compliance
- [ ] Move windows between workspaces

Some of the features listed here are also part of the EWMH and ICCM specifications, but
some are highlighted as they are usually features people expect.

### Configuring

Lucky will look for a configuration file in the following places:
1. If set, the value from `LUCKY_CONFIG` will be used;
2. If set, the value from `$XDG_CONFIG_HOME` will be used;
3. If exists, the file in `$HOME/.config/lucky` will be used;
4. If none of the above applies, the default configuration will be loaded, with a warning
