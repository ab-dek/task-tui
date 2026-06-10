# task-tui

Keyboard-driven TUI todo app. Built with [Ratatui](https://github.com/ratatui-org/ratatui).

## Features

* Organize related tasks into a single folder.
* Create multiple levels of sub-tasks.
* View percentage completions and progress.
* Fast and easy keyboard navigation.
* All data is automatically saved to and loaded from a local file.

## Keybindings
| Key | Action |
| :--- | :--- |
| <kbd>j</kbd> / <kbd>k</kbd> | Move selection down / up |
| <kbd>p</kbd> | Jump to the immediate parent of the currently selected sub-task |
| <kbd>P</kbd> | Jump to the root parent of the currently selected task tree | 
| <kbd>Enter</kbd> | Expand or collapse sub-tasks |
| <kbd>a</kbd> | Add a new folder or task |
| <kbd>A</kbd> | Add a sub-task to the currently selected task |
| <kbd>Space</kbd> | Toggle task completion (automatically updates parent task progress) |
| <kbd>d</kbd> | Delete the currently selected folder or task |
| <kbd>r</kbd> | Reset all tasks in the currently selected folder to undone |
| <kbd>y</kbd> | Confirm an action (when a popup is active) |
| <kbd>Tab</kbd> | Switch focus between the Folders pane and Tasks pane |
| <kbd>h</kbd> | Toggle the Help screen |
| <kbd>q</kbd> / <kbd>Esc</kbd> | Quit the application, or close the current popup/draft |
