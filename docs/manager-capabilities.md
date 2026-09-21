# File Manager Capabilities

The product target remains the user-facing functionality of 7-Zip 26.03 File Manager. The broader source coverage and outstanding File/Edit/View/Favorites/Tools/Help, option-page, archive-format, shortcut, and Windows-integration gaps are recorded in the scope tables in [README.md](../README.md) and [README.zh.md](../README.zh.md).

| Start-page operation | Shared behavior | Verification |
| --- | --- | --- |
| Open archive | Uses `Command::Open`, also used by the File menu and Ctrl+O. | Source inspection only. |
| Create archive | Uses `Command::Create`, also used by the File menu; opens the existing format-dependent creation form. | Source inspection only. |
| History | Existing archive open, remove, and clicked-target context menu. | Source inspection only. |
| Window movement | Centered grip marks a native title-bar drag region. | Native dragging and hit testing remain unverified. |

This layout change adds access to existing operations. It does not expand archive format support or complete the outstanding upstream feature coverage.
