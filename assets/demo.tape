# VHS tape file for sigye demo - Animation Feature Showcase
# https://github.com/charmbracelet/vhs

Output demo.gif

Set Shell "zsh"
Set FontSize 18
Set Width 1600
Set Height 800
Set Theme "Dracula"
Set TypingSpeed 50ms

# Start the clock
Type "sigye"
Enter
Sleep 1s

# Quick setup: Set Neon theme with Shifting animation
Type "s"
Sleep 300ms
# Go to Color (1x down)
Down
Sleep 100ms
# Cycle to Neon theme (12x right: Cyan->Green->Magenta->Yellow->Red->Blue->White->Rainbow->RainbowV->Warm->Cool->Ocean->Neon)
Right@100ms 12
Sleep 100ms
# Go to Animation (2x down)
Down
Sleep 100ms
Down
Sleep 100ms
# Enable Shifting (1x right)
Right
Sleep 100ms
# Go to Speed (1x down) and set to Fast
Down
Sleep 100ms
Right@100ms 2
Sleep 100ms
Enter

# SHIFTING ANIMATION - Hue rotation over time
Sleep 3s

# Switch to PULSING animation
Type "s"
Sleep 250ms
Down@80ms 3
Right
Sleep 100ms
Enter

# PULSING ANIMATION - Brightness pulses rhythmically
Sleep 3s

# Switch to WAVE animation
Type "s"
Sleep 250ms
Down@80ms 3
Right
Sleep 100ms
Enter

# WAVE ANIMATION - Color wave moves across display
Sleep 3s

# Switch to Rainbow theme with Wave for variety
Type "s"
Sleep 250ms
Down
Sleep 80ms
# Go back to Rainbow (5x left from Neon)
Left@80ms 5
Sleep 100ms
Enter

# Show Rainbow + Wave
Sleep 2.5s

# Quit
Type "q"
Sleep 200ms
