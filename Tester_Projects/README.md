## must have libadwaita-1.pc in PKG_CONFIG_PATH

pkg-config --variable pc_path pkg-config
export PKG_CONFIG_PATH=/path/to/directory/containing/libadwaita-1.pc:$PKG_CONFIG_PATH
