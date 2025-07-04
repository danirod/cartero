if host_machine.system() == 'windows'
  if get_option('win32-bundle').enabled() or get_option('win32-installer').enabled()
    meson.add_install_script('dependencies.py')
  endif

  if get_option('win32-installer').enabled()
    windows_setup = configure_file(
        input: 'installer.iss.in',
        output: 'installer.iss',
        configuration: {
        'MESON_SOURCE_ROOT': meson.project_source_root(),
        'VERSION': version,
        },
    )
    meson.add_install_script('installer.py', windows_setup)
  endif
endif