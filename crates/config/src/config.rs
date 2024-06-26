use xcb::x::KeyButMask;

use crate::keysyms::Keysym;

#[derive(Debug)]
pub struct Config {
    /// Leader key is an alias to make easy to switch the key used to execute commands, instead of
    /// using `Mod1` or `Control` you can use `Leader` which makes easy to switch the key assigned
    /// to this binding later on.
    ///
    /// Eg: `leader = "Mod1"` will bind `Mod1` as the `Leader` key
    pub(crate) leader: AvailableLeaderKeys,
    /// List of all `actions` defined in the configuration file
    pub(crate) actions: Vec<Action>,
    /// List of all `commands` defined in the configuration file
    pub(crate) commands: Vec<Command>,
    /// total workspaces to display, this is constrained to >= 1 and <= 10.
    pub(crate) workspaces: u8,
    /// the size of the border to be used by the frames
    pub(crate) border_width: u16,
    /// color to be used by borders
    pub(crate) border_color: u32,
    /// color to be used by the active client border
    pub(crate) active_border_color: u32,
    /// Altomatically focus newly created clients
    /// default: true
    pub(crate) focus_new_clients: bool,
    /// wether or not the focus should follow the cursor, focusing hovered clients
    /// default: true
    pub(crate) focus_follow_mouse: bool,
    /// commands to be executed during window manager startup
    pub(crate) startup_commands: Vec<AutoCommand>,
}

impl Config {
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn workspaces(&self) -> u8 {
        self.workspaces
    }

    pub fn border_width(&self) -> u16 {
        self.border_width
    }

    pub fn border_color(&self) -> u32 {
        self.border_color
    }

    pub fn active_border_color(&self) -> u32 {
        self.active_border_color
    }

    pub fn focus_new_clients(&self) -> bool {
        self.focus_new_clients
    }

    pub fn focus_follow_mouse(&self) -> bool {
        self.focus_follow_mouse
    }

    pub fn startup_commands(&self) -> &[AutoCommand] {
        &self.startup_commands
    }

    pub fn update(&mut self, other: Config) {
        self.leader = other.leader;
        self.actions = other.actions;
        self.commands = other.commands;
        self.workspaces = other.workspaces;
        self.border_width = other.border_width;
        self.border_color = other.border_color;
        self.active_border_color = other.active_border_color;
        self.focus_new_clients = other.focus_new_clients;
        self.focus_follow_mouse = other.focus_follow_mouse;
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            active_border_color: 0x2D4F67,
            border_color: 0x252525,
            focus_new_clients: true,
            focus_follow_mouse: true,
            border_width: 4,
            workspaces: 9,
            leader: AvailableLeaderKeys::Mod1,
            actions: vec![],
            commands: vec![],
            startup_commands: vec![],
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum AvailableLeaderKeys {
    #[default]
    Mod1,
    Shift,
    Control,
}

/// All the actions available for any given key combination
#[derive(Debug, Clone)]
pub enum AvailableActions {
    /// focus the client immediatly to the left
    FocusLeft,
    /// focus the client immediatly to the right
    FocusDown,
    /// focus the client immediatly to the bottom
    FocusUp,
    /// focus the client immediatly to the top
    FocusRight,
    /// moves a client one position to the left, shifting other clients as needed
    MoveLeft,
    /// moves a client one position to the bottom, shifting other clients as needed
    MoveDown,
    /// moves a client one position to the top, shifting other clients as needed
    MoveUp,
    /// moves a client one position to the right, shifting other clients as needed
    MoveRight,
    /// closes the focused client
    Close,
    /// Exits lucky
    Quit,
    /// Reloads the configuration file
    Reload,
    /// switches to workspace 1
    Workspace1,
    /// switches to workspace 2
    Workspace2,
    /// switches to workspace 3
    Workspace3,
    /// switches to workspace 4
    Workspace4,
    /// switches to workspace 5
    Workspace5,
    /// switches to workspace 6
    Workspace6,
    /// switches to workspace 7
    Workspace7,
    /// switches to workspace 8
    Workspace8,
    /// switches to workspace 9
    Workspace9,
    /// set focused client to be fullscreen
    Fullscreen,
    /// move the focused client to workspace 1
    MoveToWorkspace1,
    /// move the focused client to workspace 2
    MoveToWorkspace2,
    /// move the focused client to workspace 3
    MoveToWorkspace3,
    /// move the focused client to workspace 4
    MoveToWorkspace4,
    /// move the focused client to workspace 5
    MoveToWorkspace5,
    /// move the focused client to workspace 6
    MoveToWorkspace6,
    /// move the focused client to workspace 7
    MoveToWorkspace7,
    /// move the focused client to workspace 8
    MoveToWorkspace8,
    /// move the focused client to workspace 9
    MoveToWorkspace9,
}

#[derive(Debug)]
pub struct Action {
    /// Bitflag modifiers required to execute this action, example: `0x0008` maps to `Mod1`
    pub(crate) modifier: ActionModifier,
    /// The keysym used to describe this action, example: `XK_Return` matches with `Enter`
    pub(crate) key: Keysym,
    /// One of the possible actions to be performed by a key combination
    pub(crate) action: AvailableActions,
}

#[derive(Debug)]
pub struct Command {
    /// Bitflag modifiers required to execute this command, example: `0x0008` maps to `Mod1`
    pub(crate) modifier: u32,
    /// The keysym used to describe this command, example: `XK_Return` matches with `Enter`
    pub(crate) key: Keysym,
    /// The string to be spawned when this command is called
    pub(crate) command: String,
    /// the arguments to be passed to the program that will be spawned
    pub(crate) args: Vec<String>,
}

#[derive(Debug)]
pub struct AutoCommand {
    /// The string to be spawned when this command is called
    pub(crate) command: String,
    /// the arguments to be passed to the program that will be spawned
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ActionModifier(u32);

impl ActionModifier {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> u32 {
        self.0
    }
}

impl Action {
    pub fn key(&self) -> Keysym {
        self.key.clone()
    }

    pub fn modifiers(&self) -> ActionModifier {
        self.modifier
    }

    pub fn action(&self) -> AvailableActions {
        self.action.clone()
    }
}

impl Command {
    pub fn key(&self) -> Keysym {
        self.key.clone()
    }

    pub fn modifiers(&self) -> u32 {
        self.modifier
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

impl AutoCommand {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

impl From<ActionModifier> for KeyButMask {
    fn from(value: ActionModifier) -> Self {
        KeyButMask::from_bits(value.0).expect("action modifiers from config file must be valid")
    }
}
