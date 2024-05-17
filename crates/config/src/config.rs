use xcb::x::KeyButMask;

use crate::keysyms::Keysym;

#[derive(Default, Debug)]
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
    pub(crate) focus_new_clients: bool,
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

    pub fn update(&mut self, other: Config) {
        self.leader = other.leader;
        self.actions = other.actions;
        self.commands = other.commands;
        self.workspaces = other.workspaces;
        self.border_width = other.border_width;
        self.border_color = other.border_color;
        self.active_border_color = other.active_border_color;
        self.focus_new_clients = other.focus_new_clients;
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
    /// Focus the client immediatly to the left
    FocusLeft,
    /// Focus the client immediatly to the right
    FocusDown,
    /// Focus the client immediatly to the bottom
    FocusUp,
    /// Focus the client immediatly to the top
    FocusRight,
    /// Moves a client one position to the left, shifting other clients as needed
    MoveLeft,
    /// Moves a client one position to the bottom, shifting other clients as needed
    MoveDown,
    /// Moves a client one position to the top, shifting other clients as needed
    MoveUp,
    /// Moves a client one position to the right, shifting other clients as needed
    MoveRight,
    /// Closes the focused client
    Close,
    /// Reloads the configuration file
    Reload,
    /// Switches to workspace 1
    Workspace1,
    /// Switches to workspace 2
    Workspace2,
    /// Switches to workspace 3
    Workspace3,
    /// Switches to workspace 4
    Workspace4,
    /// Switches to workspace 5
    Workspace5,
    /// Switches to workspace 6
    Workspace6,
    /// Switches to workspace 7
    Workspace7,
    /// Switches to workspace 8
    Workspace8,
    /// Switches to workspace 9
    Workspace9,
    /// Switches to workspace 0
    Workspace0,
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
}

impl From<ActionModifier> for KeyButMask {
    fn from(value: ActionModifier) -> Self {
        KeyButMask::from_bits(value.0).expect("action modifiers from config file must be valid")
    }
}
