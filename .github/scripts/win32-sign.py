#!/usr/bin/env python
# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: the Cartero authors

import os
import pyotp
import subprocess

if __name__ == "__main__":
    username = os.getenv('SIGN_USERNAME')
    otp_token = os.getenv('SIGN_OTP_SECRET')
    totp = pyotp.TOTP(otp_token, digest='SHA256', issuer='Certum')
    otp = totp.now()

    cmd = [r'C:\Program Files\Certum\SimplySign Desktop\SimplySignDesktop.exe', '/autologin', username, str(otp)]
    detached = subprocess.Popen(cmd, start_new_session=True)