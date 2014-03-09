#!/bin/bash

# Script for clean before a rebuild. Since tup doesn't do this for us.

# Exclude assets dir from clean, might be keeping some scratch art there.
git clean -fdx -e assets/
# Some submodules may have generated files that mess up Tup build, need to
# clean up those separately.
git submodule foreach git clean -fdx
