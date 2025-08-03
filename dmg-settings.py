volume_name = "Roelang Installer"
format = "UDZO"  # compressed image
size = None      # auto-calculate

# Include only the app; do not include the banner as a separate file
files = [
    "dist/Roelang Installer.app"
]

# No symlink since no drag-and-drop install
symlinks = {}

# Set the banner as background (it should be ~600x400 or scale proportionally)
background = "assets/double_click.png"  # ✅ this will render behind the app icon

# Volume icon
icon = "assets/icon.icns"

# Position app icon to be visible against the background message
icon_locations = {
    "Roelang Installer.app": (200, 150)
}

# Finder window configuration
window_rect = ((200, 200), (600, 400))  # (position), (size)
show_status_bar = False
show_tab_view = False
show_toolbar = False
show_pathbar = False

code_sign_identity = None
