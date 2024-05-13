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
}

impl Config {
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
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
#[derive(Debug)]
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
    pub(crate) modifier: u32,
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

impl Action {
    pub fn key(&self) -> Keysym {
        self.key.clone()
    }

    pub fn modifiers(&self) -> u32 {
        self.modifier
    }

    pub fn action(&self) -> &AvailableActions {
        &self.action
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
