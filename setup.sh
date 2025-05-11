#!/bin/bash

export ZSAK_HOME=$PWD

if [[ $# -gt 0 ]]
then
  export PATH=$1:$PATH
else
    echo "Please, provide the path to zenohd executable"
    echo ""
    echo "USAGE:\n\t source ./setup.sh <path-to-zenohd>"
    echo ""
fi

python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install requests
