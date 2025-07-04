if host_machine.system() == 'darwin'
  if get_option('macos-app').enabled() or get_option('macos-dmg').enabled()
    if host_machine.cpu_family() == 'aarch64'
      info_plist_simplified_arch = 'arm64'
    elif host_machine.cpu_family() == 'x86_64'
      info_plist_simplified_arch = 'x86_64'
    else
      error('Unsupported host architecture')
    endif

    configure_file(
      input: 'Info.plist.in',
      output: 'Info.plist',
      configuration: {
        'BUNDLE_ID': application_id,
        'APP_VERSION': version,
        'ARCH': info_plist_simplified_arch,
      }
    )
    
    meson.add_install_script('vendoring.py', application_id)
    meson.add_install_script('packaging.py', '-i', get_option('macos-codesign-identity'))
  endif

  if get_option('macos-dmg').enabled()
    meson.add_install_script('dmg.py', '-p', get_option('macos-notary-profile'))
  endif
endif
