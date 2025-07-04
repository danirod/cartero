if host_machine.system() == 'windows'
  sign_subject = get_option('win32-sign-subject').strip()
  
  if get_option('win32-bundle').enabled() or get_option('win32-installer').enabled()
    if sign_subject != ''
      meson.add_install_script('dependencies.py', '-n', sign_subject)
    else
      meson.add_install_script('dependencies.py')
    endif
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

    if sign_subject != ''
      meson.add_install_script('installer.py', '-i', windows_setup, '-n', sign_subject)
    else
      meson.add_install_script('installer.py', '-i', windows_setup)
    endif
  endif
endif