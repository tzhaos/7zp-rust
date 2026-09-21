# Visual Verification

The source retains the existing background, inset content panel, and 48px colored toolbar artwork. The start page now arranges the brand on the left and archive commands plus history on the right, separated by a vertical rule. History rows use two lines so long directory paths do not reserve a fixed-width column. The title bar has a centered, low-contrast grip.

The layout constrains its width and height, allows command wrapping, truncates history labels, and scrolls history independently. Native results have not been inspected. Verify the 720 x 460 minimum window, larger windows, all three languages, both themes, and Windows display scaling after an explicitly requested build. Confirm the grip drags the window and does not interfere with menus or window controls.

Toolbar contraction and other pre-existing animations have not been runtime-verified as part of this change. No visual-parity claim is made.

Title-bar dropdowns now use each trigger's left edge instead of a right anchor that clamps the first menus against the window's left margin. This is a source-level correction; verify all four menu positions after a requested build.
