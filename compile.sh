#!/usr/bin/env bash

echo -e "\033[40;33mCompiling...\033[0m"

blueprint-compiler batch-compile data/ui data/ui data/ui/*.blp

echo -e "\033[42;30mDone\033[0m"
