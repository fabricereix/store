#!/bin/bash

if [[ -n $(git status -s) ]]; then
   echo "Git repo is not clean!"
   git status
   exit 1
fi


