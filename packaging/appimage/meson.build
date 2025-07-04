if host_machine.system() == 'linux' and get_option('appimage').enabled()
  desktop = '@0@.desktop'.format(application_id)
  install_symlink(desktop, install_dir: '/', pointing_to: datadir.strip('/') / 'applications' / desktop)
  # NOTE: We don't actually do this because Cartero's SVG uses some fancy
  # filters that are rendered like shit when rsvg is not in use (for instance,
  # KDE). Uncomment this the day SVG rendering does not suck in KDE:
  # icon = '@0@.svg'.format(application_id)
  # install_symlink(icon, install_dir: '/', pointing_to: datadir.strip('/') / 'icons' / 'hicolor' / 'scalable' / 'apps' / icon)
  # install_symlink('.DirIcon', install_dir: '/', pointing_to: icon)
  meson.add_install_script('package.py', application_id)
endif
