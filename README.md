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
| j / k | Move selection down / up |
| p | Jump to the immediate parent of the currently selected sub-task |
| P | Jump to the root parent of the currently selected task tree | 
| Enter | Expand or collapse sub-tasks |
| a | Add a new folder or task |
| A | Add a sub-task to the currently selected task |
| Space | Toggle task completion (automatically updates parent task progress) |
| d | Delete the currently selected folder or task |
| r | "Reset all tasks in the currently selected folder to ""undone""" |
| y | Confirm an action (when a popup is active) |
| Tab | Switch focus between the Folders pane and Tasks pane |
| h | Toggle the Help screen |
|q / Esc | Quit the application, or close the current popup/draft |
