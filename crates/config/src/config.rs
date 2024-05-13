use crate::keysyms::Keysym;

#[derive(Default, Debug)]
pub struct Config {
    pub(crate) leader: AvailableLeaderKeys,
    pub(crate) actions: Vec<Action>,
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

impl Action {
    pub fn key(&self) -> Keysym {
        self.key.clone()
    }

    pub fn modifiers(&self) -> u32 {
        self.modifier
    }
}

impl Command {
    pub fn key(&self) -> Keysym {
        self.key.clone()
    }

    pub fn modifiers(&self) -> u32 {
        self.modifier
    }
}

#[derive(Default, Debug, Clone)]
pub enum AvailableLeaderKeys {
    #[default]
    Mod1,
    Shift,
    Control,
}

#[derive(Debug)]
pub enum AvailableActions {
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    Close,
    Reload,
    Workspace1,
    Workspace2,
    Workspace3,
    Workspace4,
    Workspace5,
    Workspace6,
    Workspace7,
    Workspace8,
    Workspace9,
    Workspace0,
}

#[derive(Debug)]
pub struct Action {
    pub(crate) modifier: u32,
    pub(crate) key: Keysym,
    pub(crate) action: AvailableActions,
}

#[derive(Debug)]
pub struct Command {
    pub(crate) modifier: u32,
    pub(crate) key: Keysym,
    pub(crate) command: String,
}
